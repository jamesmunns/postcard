//! Validating schema tables, and expanding them into schemas

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, vec, vec::Vec};

use super::{
    NameRef, OwnedSchemaTable, OwnedTableData, OwnedTableNode, RawOwnedSchemaTable, VERSION,
};
use crate::schema::{OwnedData, OwnedDataModelType, OwnedNamedField, OwnedVariant};

/// Why a schema table was rejected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TableError {
    /// The table's version is not [`VERSION`].
    UnsupportedVersion(u32),
    /// A root is out of range.
    BadRoot(u32),
    /// A node refers to a node that isn't before it.
    BadChild {
        /// The index of the node
        node: usize,
        /// The index it refers to
        child: u32,
    },
    /// A name in a node is outside the names, or not on char boundaries.
    BadName {
        /// The index of the node
        node: usize,
        /// The name
        name: NameRef,
    },
    /// A node's maximum length, or an array's length, doesn't fit in a
    /// `usize` on this target.
    LenTooLarge {
        /// The index of the node
        node: usize,
    },
    /// Decoding the table would exceed a limit of the [`DecodeConfig`].
    LimitExceeded,
}

impl core::fmt::Display for TableError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported schema table version {v}"),
            Self::BadRoot(r) => write!(f, "bad root {r}"),
            Self::BadChild { node, child } => write!(f, "node {node} has bad child {child}"),
            Self::BadName { node, name } => write!(f, "node {node} has bad name {name:?}"),
            Self::LenTooLarge { node } => write!(f, "node {node} has a length too large"),
            Self::LimitExceeded => write!(f, "exceeded a configured limit"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TableError {}

/// Limits on decoding a schema table
///
/// Nodes can be shared, so a small table can decode to very large schemas: a
/// [billion laughs] style attack. Each limit is the total for all of a table's
/// roots.
///
/// [billion laughs]: https://en.wikipedia.org/wiki/Billion_laughs_attack
///
/// ```rust
/// use postcard_schema::schema::table::DecodeConfig;
///
/// let mut config = DecodeConfig::default();
/// config.max_nodes = 1 << 20;
/// ```
// schema[impl table.budget]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DecodeConfig {
    /// The maximum number of schema nodes to produce: types, variants and
    /// fields. Defaults to 65536.
    ///
    /// An array of 4 nodes would count as 5.
    pub max_nodes: usize,

    /// The maximum number of bytes of names to produce. A name counts each time
    /// it's used. Defaults to 1 MiB.
    pub max_name_bytes: usize,

    /// The maximum depth of nested types. Defaults to 128.
    ///
    /// Decoding doesn't recurse, but code that walks the decoded schemas does,
    /// once per level: key hashing, and their `Clone`, `Drop` and `Debug`. A
    /// larger limit may need a larger stack.
    pub max_depth: usize,
}

impl Default for DecodeConfig {
    fn default() -> Self {
        // defaults are guesses
        Self {
            max_nodes: 1 << 16,
            max_name_bytes: 1024 * 1024,
            max_depth: 128,
        }
    }
}

impl RawOwnedSchemaTable {
    /// Checks that the table is well formed, giving it back as an
    /// [`OwnedSchemaTable`].
    ///
    /// Every node is checked, including ones no root reaches. Each index and
    /// name is stored once in the table and checked once, so this takes time
    /// linear in the size of the table, and doesn't allocate.
    pub fn validate(self) -> Result<OwnedSchemaTable, TableError> {
        self.check(usize::MAX as u64)?;
        Ok(OwnedSchemaTable {
            version: self.version,
            roots: self.roots,
            nodes: self.nodes,
            names: self.names,
        })
    }

    /// The checks of [`RawOwnedSchemaTable::validate`], with lengths limited
    /// to `max_len` rather than `usize::MAX`, so they can be tested on any
    /// target
    pub(super) fn check(&self, max_len: u64) -> Result<(), TableError> {
        // schema[impl table.version]
        if self.version != VERSION {
            return Err(TableError::UnsupportedVersion(self.version));
        }
        // schema[impl table.root]
        for &root in &self.roots {
            if root as usize >= self.nodes.len() {
                return Err(TableError::BadRoot(root));
            }
        }
        // schema[impl table.reject]
        // schema[impl table.accept]
        for (i, node) in self.nodes.iter().enumerate() {
            let child = |c: u32| {
                // schema[impl table.child.order]
                if (c as usize) < i {
                    Ok(())
                } else {
                    Err(TableError::BadChild { node: i, child: c })
                }
            };
            // schema[impl table.name]
            let name = |n: NameRef| {
                let start = n.offset as usize;
                match start
                    .checked_add(n.len as usize)
                    .and_then(|end| self.names.get(start..end))
                {
                    Some(_) => Ok(()),
                    None => Err(TableError::BadName { node: i, name: n }),
                }
            };
            // schema[impl table.max-len]
            let len = |n: Option<u64>| match n {
                Some(n) if n > max_len => Err(TableError::LenTooLarge { node: i }),
                _ => Ok(()),
            };
            let data = |d: &OwnedTableData| match d {
                OwnedTableData::Unit => Ok(()),
                OwnedTableData::Newtype(c) => child(*c),
                OwnedTableData::Tuple(cs) => cs.iter().try_for_each(|&c| child(c)),
                OwnedTableData::Struct(fs) => fs.iter().try_for_each(|f| {
                    name(f.name)?;
                    child(f.ty)
                }),
            };
            match node {
                OwnedTableNode::String { max_len } | OwnedTableNode::ByteArray { max_len } => {
                    len(*max_len)?
                }
                OwnedTableNode::Option(c) => child(*c)?,
                OwnedTableNode::Seq { element, max_len } => {
                    child(*element)?;
                    len(*max_len)?;
                }
                OwnedTableNode::Tuple(cs) => cs.iter().try_for_each(|&c| child(c))?,
                OwnedTableNode::Map { key, val, max_len } => {
                    child(*key)?;
                    child(*val)?;
                    len(*max_len)?;
                }
                OwnedTableNode::Struct { name: n, data: d } => {
                    name(*n)?;
                    data(d)?;
                }
                OwnedTableNode::Enum { name: n, variants } => {
                    name(*n)?;
                    for v in variants {
                        name(v.name)?;
                        data(&v.data)?;
                    }
                }
                OwnedTableNode::Array { element, len: l } => {
                    child(*element)?;
                    len(Some(*l))?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl OwnedSchemaTable {
    /// Expands each root into a schema, with the default [`DecodeConfig`].
    pub fn decode(&self) -> Result<Vec<OwnedDataModelType>, TableError> {
        self.decode_with_config(&DecodeConfig::default())
    }

    /// Expands each root into a schema.
    ///
    /// The size and depth of the schemas are worked out from the table
    /// first, without recursion, so a table that exceeds a limit is rejected
    /// before anything is expanded.
    ///
    /// Returns [`TableError::LimitExceeded`] if the schemas exceed any of
    /// `config`'s limits. A `max_depth` of 0 rejects every table.
    // schema[impl table.meaning]
    pub fn decode_with_config(
        &self,
        config: &DecodeConfig,
    ) -> Result<Vec<OwnedDataModelType>, TableError> {
        let uses = self.uses();
        self.check_size(&uses, config)?;
        Ok(self.expand(uses))
    }

    /// The number of times each node is used by the roots, counting a node once
    /// for each place it's used in a node that is used itself. Children come
    /// first, so walking backwards counts each node's users before the node.
    fn uses(&self) -> Vec<usize> {
        let mut uses = vec![0; self.nodes.len()];
        for &r in &self.roots {
            uses[r as usize] += 1;
        }
        for i in (0..self.nodes.len()).rev() {
            if uses[i] != 0 {
                children(&self.nodes[i], |c| uses[c as usize] += 1);
            }
        }
        uses
    }

    /// Checks the size and depth of every used node's schema, and the total of
    /// the roots, against `config`.
    fn check_size(&self, uses: &[usize], config: &DecodeConfig) -> Result<(), TableError> {
        let limit = Size {
            nodes: config.max_nodes,
            name_bytes: config.max_name_bytes,
            depth: config.max_depth,
        };
        let mut sizes = vec![Size::ZERO; self.nodes.len()];
        for (i, node) in self.nodes.iter().enumerate() {
            if uses[i] != 0 {
                sizes[i] = self
                    .size(node, &sizes)
                    .filter(|s| s.within(&limit))
                    .ok_or(TableError::LimitExceeded)?;
            }
        }
        let mut total = Size::ZERO;
        for &r in &self.roots {
            total = total
                .and(&sizes[r as usize])
                .ok_or(TableError::LimitExceeded)?;
        }
        if total.within(&limit) {
            Ok(())
        } else {
            Err(TableError::LimitExceeded)
        }
    }

    /// The size of `node`'s schema, given the sizes of the nodes before it, or
    /// `None` if it overflows
    fn size(&self, node: &OwnedTableNode, sizes: &[Size]) -> Option<Size> {
        use OwnedTableNode as N;
        let of = |c: &u32| Some(sizes[*c as usize]);
        let name = |n: &NameRef| Size {
            nodes: 0,
            name_bytes: n.len as usize,
            depth: 0,
        };
        // A variant or field is one node, and its name
        let named = |n: &NameRef| Size {
            nodes: 1,
            ..name(n)
        };
        let data = |d: &OwnedTableData| match d {
            OwnedTableData::Unit => Some(Size::ZERO),
            OwnedTableData::Newtype(c) => of(c),
            OwnedTableData::Tuple(cs) => Size::sum(cs.iter().map(of)),
            OwnedTableData::Struct(fs) => {
                Size::sum(fs.iter().map(|f| named(&f.name).and(&of(&f.ty)?)))
            }
        };
        let inner = match node {
            N::Option(c) | N::Seq { element: c, .. } => of(c)?,
            N::Tuple(cs) => Size::sum(cs.iter().map(of))?,
            N::Map { key, val, .. } => of(key)?.and(&of(val)?)?,
            N::Struct { name: n, data: d } => name(n).and(&data(d)?)?,
            N::Enum { name: n, variants } => {
                let vs = variants.iter().map(|v| named(&v.name).and(&data(&v.data)?));
                name(n).and(&Size::sum(vs)?)?
            }
            N::Array { element, len } => {
                if *len == 0 {
                    Size::ZERO
                } else {
                    // validation checked that `len` fits
                    of(element)?.times(*len as usize)?
                }
            }
            _ => Size::ZERO,
        };
        // The node itself, one level above what it contains
        Some(Size {
            nodes: inner.nodes.checked_add(1)?,
            name_bytes: inner.name_bytes,
            depth: inner.depth.checked_add(1)?,
        })
    }

    /// Expands the used nodes in order, so each node's children are already
    /// expanded.
    fn expand(&self, uses: Vec<usize>) -> Vec<OwnedDataModelType> {
        let mut x = Expanded {
            t: self,
            done: vec![None; self.nodes.len()],
            uses,
        };
        for i in 0..self.nodes.len() {
            if x.uses[i] != 0 {
                x.done[i] = Some(x.node(&self.nodes[i]));
            }
        }
        self.roots.iter().map(|&r| x.take(r)).collect()
    }
}

/// The nodes expanded so far, and how many more times each is used
struct Expanded<'a> {
    t: &'a OwnedSchemaTable,
    done: Vec<Option<OwnedDataModelType>>,
    uses: Vec<usize>,
}

impl Expanded<'_> {
    /// An expanded node: moved out for its last use, and cloned before that
    fn take(&mut self, idx: u32) -> OwnedDataModelType {
        let i = idx as usize;
        self.uses[i] -= 1;
        let ty = if self.uses[i] == 0 {
            self.done[i].take()
        } else {
            self.done[i].clone()
        };
        ty.expect("a used node is expanded before its users")
    }

    fn boxed(&mut self, idx: u32) -> Box<OwnedDataModelType> {
        Box::new(self.take(idx))
    }

    fn tys(&mut self, idxs: &[u32]) -> Box<[OwnedDataModelType]> {
        idxs.iter().map(|&c| self.take(c)).collect()
    }

    fn name(&self, name: &NameRef) -> Box<str> {
        let start = name.offset as usize;
        self.t.names[start..start + name.len as usize].into()
    }

    fn node(&mut self, node: &OwnedTableNode) -> OwnedDataModelType {
        use OwnedDataModelType as T;
        use OwnedTableNode as N;
        // validation checked that every maximum length fits
        let max = |m: &Option<u64>| m.map(|n| n as usize);
        match node {
            N::Bool => T::Bool,
            N::I8 => T::I8,
            N::U8 => T::U8,
            N::I16 => T::I16,
            N::I32 => T::I32,
            N::I64 => T::I64,
            N::I128 => T::I128,
            N::U16 => T::U16,
            N::U32 => T::U32,
            N::U64 => T::U64,
            N::U128 => T::U128,
            N::F32 => T::F32,
            N::F64 => T::F64,
            N::Char => T::Char,
            N::String { max_len } => T::String {
                max_len: max(max_len),
            },
            N::ByteArray { max_len } => T::ByteArray {
                max_len: max(max_len),
            },
            N::Option(c) => T::Option(self.boxed(*c)),
            N::Unit => T::Unit,
            N::Seq { element, max_len } => T::Seq {
                element: self.boxed(*element),
                max_len: max(max_len),
            },
            N::Tuple(cs) => T::Tuple(self.tys(cs)),
            N::Map { key, val, max_len } => T::Map {
                key: self.boxed(*key),
                val: self.boxed(*val),
                max_len: max(max_len),
            },
            N::Struct { name, data } => T::Struct {
                name: self.name(name),
                data: self.data(data),
            },
            N::Enum { name, variants } => T::Enum {
                name: self.name(name),
                variants: variants
                    .iter()
                    .map(|v| OwnedVariant {
                        name: self.name(&v.name),
                        data: self.data(&v.data),
                    })
                    .collect(),
            },
            // schema[impl table.array]
            N::Array { element, len } => T::Tuple(match *len {
                0 => Box::new([]),
                // validation checked that `len` fits
                len => core::iter::repeat_n(self.take(*element), len as usize).collect(),
            }),
        }
    }

    fn data(&mut self, data: &OwnedTableData) -> OwnedData {
        match data {
            OwnedTableData::Unit => OwnedData::Unit,
            OwnedTableData::Newtype(c) => OwnedData::Newtype(self.boxed(*c)),
            OwnedTableData::Tuple(cs) => OwnedData::Tuple(self.tys(cs)),
            OwnedTableData::Struct(fs) => OwnedData::Struct(
                fs.iter()
                    .map(|f| OwnedNamedField {
                        name: self.name(&f.name),
                        ty: self.take(f.ty),
                    })
                    .collect(),
            ),
        }
    }
}

/// Calls `f` with each node index in `node` that is expanded with it. An
/// `Array` of no copies doesn't expand its element.
fn children(node: &OwnedTableNode, mut f: impl FnMut(u32)) {
    use OwnedTableNode as N;
    let data = |d: &OwnedTableData, f: &mut dyn FnMut(u32)| match d {
        OwnedTableData::Unit => {}
        OwnedTableData::Newtype(c) => f(*c),
        OwnedTableData::Tuple(cs) => cs.iter().for_each(|&c| f(c)),
        OwnedTableData::Struct(fs) => fs.iter().for_each(|fd| f(fd.ty)),
    };
    match node {
        N::Option(c) | N::Seq { element: c, .. } => f(*c),
        N::Tuple(cs) => cs.iter().for_each(|&c| f(c)),
        N::Map { key, val, .. } => {
            f(*key);
            f(*val);
        }
        N::Struct { data: d, .. } => data(d, &mut f),
        N::Enum { variants, .. } => variants.iter().for_each(|v| data(&v.data, &mut f)),
        N::Array { element, len } if *len != 0 => f(*element),
        _ => {}
    }
}

/// The size of a schema, as [`DecodeConfig`] counts it
#[derive(Debug, Clone, Copy)]
struct Size {
    nodes: usize,
    name_bytes: usize,
    depth: usize,
}

impl Size {
    const ZERO: Size = Size {
        nodes: 0,
        name_bytes: 0,
        depth: 0,
    };

    /// Two schemas side by side: the totals, and the greater depth
    fn and(&self, other: &Size) -> Option<Size> {
        Some(Size {
            nodes: self.nodes.checked_add(other.nodes)?,
            name_bytes: self.name_bytes.checked_add(other.name_bytes)?,
            depth: self.depth.max(other.depth),
        })
    }

    fn sum(sizes: impl IntoIterator<Item = Option<Size>>) -> Option<Size> {
        sizes.into_iter().try_fold(Size::ZERO, |a, b| a.and(&b?))
    }

    /// `n` copies of a schema side by side
    fn times(&self, n: usize) -> Option<Size> {
        Some(Size {
            nodes: self.nodes.checked_mul(n)?,
            name_bytes: self.name_bytes.checked_mul(n)?,
            depth: self.depth,
        })
    }

    fn within(&self, limit: &Size) -> bool {
        self.nodes <= limit.nodes
            && self.name_bytes <= limit.name_bytes
            && self.depth <= limit.depth
    }
}
