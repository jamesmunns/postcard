//! Schema tables built or received at runtime

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};
#[cfg(feature = "std")]
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    array_of,
    intern::{self, Interner},
    to_u32, DecodeConfig, NameRef, SchemaTableRef, TableData, TableError, TableField, TableNode,
    TableVariant, VERSION,
};
use crate::schema::{OwnedData, OwnedDataModelType, OwnedNamedField, OwnedVariant};

/// An owned schema table, which may not be valid
///
/// This is a table as it's encoded, with public fields, so one can be built or
/// inspected by hand. [`RawOwnedSchemaTable::validate`] checks it, giving an
/// [`OwnedSchemaTable`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "SchemaTable")]
pub struct RawOwnedSchemaTable {
    /// Always [`VERSION`] in a valid table
    pub version: u32,
    /// The node index of each of the table's types
    pub roots: Vec<u32>,
    /// Types, each after all of the types it contains
    pub nodes: Vec<OwnedTableNode>,
    /// The text of every name, referred to by [`NameRef`]s
    pub names: String,
}

///  A valid owned schema table, such as one received from a peer
///
/// Deserializing a table validates it, and fails with the [`TableError`]'s
/// message if it isn't valid. Its parts can be read but not changed;
/// [`OwnedSchemaTable::into_raw`] gives them back as a
/// [`RawOwnedSchemaTable`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "SchemaTable", try_from = "RawOwnedSchemaTable")]
pub struct OwnedSchemaTable {
    pub(super) version: u32,
    pub(super) roots: Vec<u32>,
    pub(super) nodes: Vec<OwnedTableNode>,
    pub(super) names: String,
}

impl OwnedSchemaTable {
    /// The node index of each of the table's types
    pub fn roots(&self) -> &[u32] {
        &self.roots
    }

    /// The table's types, each after all of the types it contains
    pub fn nodes(&self) -> &[OwnedTableNode] {
        &self.nodes
    }

    /// The text of every name, referred to by [`NameRef`]s
    pub fn names(&self) -> &str {
        &self.names
    }

    /// The table's parts, which can then be changed
    pub fn into_raw(self) -> RawOwnedSchemaTable {
        self.into()
    }
}

impl From<OwnedSchemaTable> for RawOwnedSchemaTable {
    fn from(t: OwnedSchemaTable) -> Self {
        RawOwnedSchemaTable {
            version: t.version,
            roots: t.roots,
            nodes: t.nodes,
            names: t.names,
        }
    }
}

impl TryFrom<RawOwnedSchemaTable> for OwnedSchemaTable {
    type Error = TableError;

    fn try_from(t: RawOwnedSchemaTable) -> Result<Self, TableError> {
        t.validate()
    }
}

/// One type in an [`OwnedSchemaTable`]
///
/// Each `u32` is the index of a node, which is before this one. The variants
/// are in the same order as [`TableNode`]'s, which gives their discriminants.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename = "TableNode")]
#[allow(missing_docs)]
pub enum OwnedTableNode {
    Bool,
    I8,
    U8,
    I16,
    I32,
    I64,
    I128,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
    String {
        max_len: Option<u64>,
    },
    ByteArray {
        max_len: Option<u64>,
    },
    Option(u32),
    Unit,
    Seq {
        element: u32,
        max_len: Option<u64>,
    },
    Tuple(Vec<u32>),
    Map {
        key: u32,
        val: u32,
        max_len: Option<u64>,
    },
    Struct {
        name: NameRef,
        data: OwnedTableData,
    },
    Enum {
        name: NameRef,
        variants: Vec<OwnedTableVariant>,
    },
    Array {
        element: u32,
        len: u64,
    },
}

/// The contents of a struct or variant in an [`OwnedSchemaTable`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename = "TableData")]
#[allow(missing_docs)]
pub enum OwnedTableData {
    Unit,
    Newtype(u32),
    Tuple(Vec<u32>),
    Struct(Vec<TableField>),
}

/// A variant of an enum in an [`OwnedSchemaTable`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename = "TableVariant")]
pub struct OwnedTableVariant {
    /// The variant's name
    pub name: NameRef,
    /// The variant's contents
    pub data: OwnedTableData,
}

impl OwnedSchemaTable {
    /// Builds a table of `roots`, within the default [`DecodeConfig`]
    ///
    /// Each distinct type is a root once, in the order first given.
    ///
    /// This gives the same table as [`schema_table!`](crate::schema_table) does
    /// for the same schemas.
    pub fn new(roots: &[OwnedDataModelType]) -> Result<Self, TableError> {
        Self::new_with_config(roots, &DecodeConfig::default())
    }

    /// Builds a table of `roots`, if they are within `config`'s limits
    ///
    /// The limits are counted as decoding counts them, for all of `roots`
    /// together, so the table decodes with the same config. They are checked
    /// before building, without recursion, and building recurses once per level
    /// of nested types.
    ///
    /// Building uses about 70 bytes of scratch space for each byte of names,
    /// counting each use, so `max_name_bytes` limits that too.
    ///
    /// Panics if the table would have `u32::MAX` or more nodes or name bytes.
    pub fn new_with_config(
        roots: &[OwnedDataModelType],
        config: &DecodeConfig,
    ) -> Result<Self, TableError> {
        // The space for names is an upper bound, assuming none are reused.
        let name_bytes = name_bytes(roots, config).ok_or(TableError::LimitExceeded)?;
        let mut names = vec![0; name_bytes];
        let mut states = vec![intern::EMPTY_STATE; intern::states_for(name_bytes)];
        let mut edges = vec![intern::EMPTY_EDGE; intern::edges_for(name_bytes)];
        let mut b = Builder {
            nodes: Vec::new(),
            index: BTreeMap::new(),
            names: Interner::new(&mut names, &mut states, &mut edges),
        };
        let mut seen = BTreeMap::new();
        let roots = roots
            .iter()
            .map(|r| b.ty(r))
            .filter(|&idx| seen.insert(idx, ()).is_none())
            .collect();
        let (nodes, n_names) = (b.nodes, b.names.n_names);
        names.truncate(n_names);
        Ok(OwnedSchemaTable {
            version: VERSION,
            roots,
            nodes,
            // Names are appended whole, so the bytes are valid UTF-8.
            names: String::from_utf8(names).expect("names are UTF-8"),
        })
    }
}

impl From<SchemaTableRef<'_>> for RawOwnedSchemaTable {
    fn from(t: SchemaTableRef<'_>) -> Self {
        let data = |d: TableData| match d {
            TableData::Unit => OwnedTableData::Unit,
            TableData::Newtype(n) => OwnedTableData::Newtype(n),
            TableData::Tuple(s) => OwnedTableData::Tuple(t.children_of(s).into()),
            TableData::Struct(s) => OwnedTableData::Struct(t.fields_of(s).into()),
        };
        let node = |n: &TableNode| match *n {
            TableNode::Bool => OwnedTableNode::Bool,
            TableNode::I8 => OwnedTableNode::I8,
            TableNode::U8 => OwnedTableNode::U8,
            TableNode::I16 => OwnedTableNode::I16,
            TableNode::I32 => OwnedTableNode::I32,
            TableNode::I64 => OwnedTableNode::I64,
            TableNode::I128 => OwnedTableNode::I128,
            TableNode::U16 => OwnedTableNode::U16,
            TableNode::U32 => OwnedTableNode::U32,
            TableNode::U64 => OwnedTableNode::U64,
            TableNode::U128 => OwnedTableNode::U128,
            TableNode::F32 => OwnedTableNode::F32,
            TableNode::F64 => OwnedTableNode::F64,
            TableNode::Char => OwnedTableNode::Char,
            TableNode::String { max_len } => OwnedTableNode::String { max_len },
            TableNode::ByteArray { max_len } => OwnedTableNode::ByteArray { max_len },
            TableNode::Option(n) => OwnedTableNode::Option(n),
            TableNode::Unit => OwnedTableNode::Unit,
            TableNode::Seq { element, max_len } => OwnedTableNode::Seq { element, max_len },
            TableNode::Tuple(s) => OwnedTableNode::Tuple(t.children_of(s).into()),
            TableNode::Map { key, val, max_len } => OwnedTableNode::Map { key, val, max_len },
            TableNode::Struct { name, data: d } => OwnedTableNode::Struct {
                name,
                data: data(d),
            },
            TableNode::Enum { name, variants } => OwnedTableNode::Enum {
                name,
                variants: t
                    .variants_of(variants)
                    .iter()
                    .map(|v: &TableVariant| OwnedTableVariant {
                        name: v.name,
                        data: data(v.data),
                    })
                    .collect(),
            },
            TableNode::Array { element, len } => OwnedTableNode::Array { element, len },
        };
        RawOwnedSchemaTable {
            version: t.version,
            roots: t.roots.into(),
            nodes: t.nodes.iter().map(node).collect(),
            names: t.names.into(),
        }
    }
}

/// The total length of the names in `roots`, counting each use, or `None` if
/// `roots` exceed `config`'s limits
fn name_bytes(roots: &[OwnedDataModelType], config: &DecodeConfig) -> Option<usize> {
    use OwnedDataModelType as T;
    /// Something still to count, with the depth of its type
    enum Item<'a> {
        Ty(&'a OwnedDataModelType, usize),
        Data(&'a OwnedData, usize),
    }
    let (mut nodes, mut name_bytes) = (0usize, 0usize);
    let mut count = |n: usize, names: usize| {
        nodes = nodes.checked_add(n)?;
        name_bytes = name_bytes.checked_add(names)?;
        (nodes <= config.max_nodes && name_bytes <= config.max_name_bytes).then_some(())
    };
    let mut stack: Vec<Item<'_>> = roots.iter().map(|r| Item::Ty(r, 1)).collect();
    while let Some(item) = stack.pop() {
        match item {
            Item::Ty(ty, depth) => {
                if depth > config.max_depth {
                    return None;
                }
                count(1, 0)?;
                let inner = depth.saturating_add(1);
                match ty {
                    T::Option(t) | T::Seq { element: t, .. } => stack.push(Item::Ty(t, inner)),
                    T::Tuple(ts) => stack.extend(ts.iter().map(|t| Item::Ty(t, inner))),
                    T::Map { key, val, .. } => {
                        stack.push(Item::Ty(key, inner));
                        stack.push(Item::Ty(val, inner));
                    }
                    T::Struct { name, data } => {
                        count(0, name.len())?;
                        stack.push(Item::Data(data, depth));
                    }
                    T::Enum { name, variants } => {
                        count(0, name.len())?;
                        for v in variants.iter() {
                            count(1, v.name.len())?;
                            stack.push(Item::Data(&v.data, depth));
                        }
                    }
                    _ => {}
                }
            }
            // The data of a type at `depth`
            Item::Data(data, depth) => {
                let inner = depth.saturating_add(1);
                match data {
                    OwnedData::Unit => {}
                    OwnedData::Newtype(t) => stack.push(Item::Ty(t, inner)),
                    OwnedData::Tuple(ts) => stack.extend(ts.iter().map(|t| Item::Ty(t, inner))),
                    OwnedData::Struct(fs) => {
                        for f in fs.iter() {
                            count(1, f.name.len())?;
                            stack.push(Item::Ty(&f.ty, inner));
                        }
                    }
                }
            }
        }
    }
    Some(name_bytes)
}

/// Builds a table the way the const builder does, so both give the same table.
/// Nodes are deduplicated with a map and names with [`Interner`], so building
/// takes time linear in the size of the schemas, up to the map's
/// logarithm.
// schema[impl table.dedup]
struct Builder<'a> {
    nodes: Vec<OwnedTableNode>,
    index: BTreeMap<OwnedTableNode, u32>,
    names: Interner<'a>,
}

impl Builder<'_> {
    fn node(&mut self, node: OwnedTableNode) -> u32 {
        if let Some(&idx) = self.index.get(&node) {
            return idx;
        }
        let idx = to_u32(self.nodes.len());
        self.index.insert(node.clone(), idx);
        self.nodes.push(node);
        idx
    }

    fn name(&mut self, name: &str) -> NameRef {
        self.names.intern(name)
    }

    fn tys(&mut self, tys: &[OwnedDataModelType]) -> Vec<u32> {
        tys.iter().map(|t| self.ty(t)).collect()
    }

    fn ty(&mut self, ty: &OwnedDataModelType) -> u32 {
        use OwnedDataModelType as T;
        use OwnedTableNode as N;
        let max = |m: &Option<usize>| m.map(|n| n as u64);
        let node = match ty {
            T::Bool => N::Bool,
            T::I8 => N::I8,
            T::U8 => N::U8,
            T::I16 => N::I16,
            T::I32 => N::I32,
            T::I64 => N::I64,
            T::I128 => N::I128,
            T::U16 => N::U16,
            T::U32 => N::U32,
            T::U64 => N::U64,
            T::U128 => N::U128,
            T::F32 => N::F32,
            T::F64 => N::F64,
            T::Char => N::Char,
            T::String { max_len } => N::String {
                max_len: max(max_len),
            },
            T::ByteArray { max_len } => N::ByteArray {
                max_len: max(max_len),
            },
            T::Option(inner) => N::Option(self.ty(inner)),
            T::Unit => N::Unit,
            T::Seq { element, max_len } => N::Seq {
                element: self.ty(element),
                max_len: max(max_len),
            },
            T::Tuple(tys) => {
                let idxs = self.tys(tys);
                match array_of(&idxs) {
                    Some((element, len)) => N::Array { element, len },
                    None => N::Tuple(idxs),
                }
            }
            T::Map { key, val, max_len } => N::Map {
                key: self.ty(key),
                val: self.ty(val),
                max_len: max(max_len),
            },
            T::Struct { name, data } => N::Struct {
                name: self.name(name),
                data: self.data(data),
            },
            T::Enum { name, variants } => {
                let name = self.name(name);
                let variants = variants.iter().map(|v| self.variant(v)).collect();
                N::Enum { name, variants }
            }
        };
        self.node(node)
    }

    fn variant(&mut self, v: &OwnedVariant) -> OwnedTableVariant {
        OwnedTableVariant {
            name: self.name(&v.name),
            data: self.data(&v.data),
        }
    }

    fn field(&mut self, f: &OwnedNamedField) -> TableField {
        TableField {
            name: self.name(&f.name),
            ty: self.ty(&f.ty),
        }
    }

    fn data(&mut self, data: &OwnedData) -> OwnedTableData {
        match data {
            OwnedData::Unit => OwnedTableData::Unit,
            OwnedData::Newtype(inner) => OwnedTableData::Newtype(self.ty(inner)),
            OwnedData::Tuple(tys) => OwnedTableData::Tuple(self.tys(tys)),
            OwnedData::Struct(fields) => {
                OwnedTableData::Struct(fields.iter().map(|f| self.field(f)).collect())
            }
        }
    }
}
