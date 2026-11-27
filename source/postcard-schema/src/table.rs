//! Schema tables: flat, interned schemas
//!
//! A schema table holds the same information as one or more [`DataModelType`]s,
//! with no references. Each distinct type is stored once, as a node, and nodes
//! refer to the types they contain by index. Every name is interned into one
//! string. A table is an ordinary, non-recursive type, which implements
//! postcard-schema's `Schema` trait, as well as serde's `Serialize` and
//! `Deserialize` traits.
//!
//! Each node comes after all of the nodes it refers to, so following an index
//! always points backwards, and a table can't contain a cycle. The roots are
//! the node indices of the types the table was built from.
//!
//! A table has two forms:
//!
//! * [`SchemaTable`], built at compile time with [`schema_table!`]. It stores
//!   its lists of children, variants and fields in arrays of their own,
//!   referred to by [`Span`]s, so that equal lists are stored once.
//! * [`OwnedSchemaTable`], with the `alloc` or `std` feature, which is how a
//!   table is received, validated and decoded. Its nodes hold their lists
//!   directly, as the encoding does.
//!
//! Both have the same encoding and the same schema. In the encoding, each node
//! holds its own variants, fields and tuple elements.
//!
//! Deserializing an [`OwnedSchemaTable`] validates it. [`RawOwnedSchemaTable`]
//! is a table that may not be valid, such as one built by hand.
//! [`TableLimits`] bounds the schemas that decoding a table, or building one at
//! runtime, can produce. The nodes, and the lists and names they refer to, are
//! in [`node`].
//!
//! [`schema_table_bytes!`] goes one step further, and serializes the table in
//! the postcard format as a byte array at compile time, so it can be sent with
//! no work at runtime.
//!
//! # Naming a table's type
//!
//! [`schema_table!`] evaluates to a table, so it can't name the table's type,
//! which depends on the lengths of the table. A `static`, or a type that hold
//! a table, needs the type written out. [`impl_schema_table!`] builds the table
//! into associated consts of a type you give it, and [`schema_table_type!`]
//! names the type of that table:
//!
//! ```
//! use postcard_schema::{impl_schema_table, schema_table, schema_table_type};
//!
//! struct Helper;
//! impl_schema_table!(Helper, u8, Option<u8>);
//!
//! static TABLE: schema_table_type!(Helper) = Helper::SCHEMA_TABLE;
//!
//! assert_eq!(TABLE, schema_table!(u8, Option<u8>));
//! ```
//!
//! [`schema_table!`]: crate::schema_table
//! [`impl_schema_table!`]: crate::impl_schema_table
//! [`schema_table_type!`]: crate::schema_table_type
//! [`schema_table_bytes!`]: crate::schema_table_bytes

use crate::schema::{Data, DataModelType, NamedField, Variant};
use intern::{Interner, EMPTY_EDGE, EMPTY_STATE};
use node::{NameRef, Span, TableData, TableField, TableNode, TableVariant};
#[cfg(any(feature = "std", feature = "alloc"))]
use node::{OwnedTableData, OwnedTableNode, OwnedTableVariant};
use serde::{
    ser::{SerializeStruct, SerializeStructVariant},
    Serialize, Serializer,
};

/// The current version of the table format
pub const VERSION: u32 = 1;

// Building at compile time takes three steps, as array lengths have to be known
// in const context:
//
// 1. `scratch_lengths()` counts nodes, lists and name bytes without
//    deduplication, giving an upper bound on the space needed.
// 2. `TableScratch::new()` builds the deduplicated table into scratch space of
//    that size, and `TableScratch::lengths()` reports how much of it was used.
// 3. `SchemaTable::new()` copies the used part into right-sized arrays.
//
// The macros do all three. The steps are public for the macros, but hidden from
// the docs; `impl_schema_table!` is the API for naming a table's type.

// ---- Step one: upper bounds ----

/// Counts the space needed to flatten types, without deduplication.
struct Counter {
    nodes: usize,
    children: usize,
    variants: usize,
    fields: usize,
    names: usize,
}

impl Counter {
    /// Increment the max space for storing this type with no deduplication.
    const fn ty(&mut self, ty: &DataModelType) {
        self.nodes += 1;
        match ty {
            DataModelType::Option(inner) => self.ty(inner),
            DataModelType::Seq { element, .. } => self.ty(element),
            DataModelType::Tuple(tys) => self.tys(tys),
            DataModelType::Map { key, val, .. } => {
                self.ty(key);
                self.ty(val);
            }
            DataModelType::Struct { name, data } => {
                self.names += name.len();
                self.data(data);
            }
            DataModelType::Enum { name, variants } => {
                self.names += name.len();
                self.variants += variants.len();
                let mut i = 0;
                while i < variants.len() {
                    self.names += variants[i].name.len();
                    self.data(&variants[i].data);
                    i += 1;
                }
            }
            _ => {}
        }
    }

    /// Increment the max space for all of the given tys, with no deduplication
    const fn tys(&mut self, tys: &[&DataModelType]) {
        self.children += tys.len();
        let mut i = 0;
        while i < tys.len() {
            self.ty(tys[i]);
            i += 1;
        }
    }

    /// Increment the max space for storing this data with no deduplication.
    const fn data(&mut self, data: &Data) {
        match data {
            Data::Unit => {}
            Data::Newtype(inner) => self.ty(inner),
            Data::Tuple(tys) => self.tys(tys),
            Data::Struct(fields) => {
                self.fields += fields.len();
                let mut i = 0;
                while i < fields.len() {
                    self.names += fields[i].name.len();
                    self.ty(fields[i].ty);
                    i += 1;
                }
            }
        }
    }
}

/// The array lengths of a [`TableScratch`] or [`SchemaTable`]
///
/// [`scratch_lengths()`] gives an upper bound, to size a [`TableScratch`].
/// [`TableScratch::lengths()`] gives the exact lengths, to size a
/// [`SchemaTable`]. The fields are in the order of the const parameters of both.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableLengths {
    /// The number of roots, which is the number of inputs
    pub roots: usize,
    /// The number of nodes
    pub nodes: usize,
    /// The number of entries in the children array
    pub children: usize,
    /// The number of entries in the variants array
    pub variants: usize,
    /// The number of entries in the fields array
    pub fields: usize,
    /// The number of bytes of interned names
    pub names: usize,
}

impl TableLengths {
    /// The `STATES` argument of [`TableScratch::new()`], for these lengths.
    ///
    /// Call this on the lengths from [`scratch_lengths()`]; only `names` is
    /// used.
    pub const fn interner_states(&self) -> usize {
        intern::states_for(self.names)
    }

    /// The `EDGES` argument of [`TableScratch::new()`], for these lengths.
    ///
    /// Call this on the lengths from [`scratch_lengths()`]; only `names` is
    /// used.
    pub const fn interner_edges(&self) -> usize {
        intern::edges_for(self.names)
    }
}

/// The scratch space needed to build a table of `roots`: step one of building
/// a table.
///
/// This is an upper bound, assuming nothing can be deduplicated. The space for
/// each list is doubled, as a list being collected is kept on a stack at the
/// end while it's copied to the start. Use it for the const parameters of
/// [`TableScratch`], and for [`TableLengths::interner_states()`] and
/// [`TableLengths::interner_edges()`].
///
/// The cost is linear in the size of the types, with no deduplication.
#[doc(hidden)]
pub const fn scratch_lengths(roots: &[&DataModelType]) -> TableLengths {
    let mut c = Counter {
        nodes: 0,
        children: 0,
        variants: 0,
        fields: 0,
        names: 0,
    };
    let mut i = 0;
    while i < roots.len() {
        c.ty(roots[i]);
        i += 1;
    }
    TableLengths {
        roots: roots.len(),
        nodes: c.nodes,
        children: 2 * c.children,
        variants: 2 * c.variants,
        fields: 2 * c.fields,
        names: c.names,
    }
}

// ---- Step two: interning into scratch space ----

/// Writes deduplicated nodes, lists and names into scratch space.
///
/// Spans are written to the start of each array, and a stack starting at the
/// end is used for the span being collected.
// TODO: Node and list search is quadratic. probe-rs's interface uses about half
// the const eval budget today.
// schema[impl table.dedup]
struct Writer<'a> {
    nodes: &'a mut [TableNode],
    n_nodes: usize,
    children: &'a mut [u32],
    n_children: usize,
    c_stack: usize,
    variants: &'a mut [TableVariant],
    n_variants: usize,
    v_stack: usize,
    fields: &'a mut [TableField],
    n_fields: usize,
    f_stack: usize,
    names: Interner<'a>,
}

/// Defines a push and a pop for one of the [`Writer`]'s lists.
///
/// This is... regrettable. We don't have const traits, so defining a generic
/// "stack"/"list" and using it for each of the things we're collecting isn't
/// really an option, so instead we stamp out things using a macro.
macro_rules! list_ops {
    ($push:ident, $pop:ident, $list:ident, $n:ident, $stack:ident, $ty:ty, $eq:ident) => {
        /// Pushes an item onto the stack, for the span being collected.
        const fn $push(&mut self, item: $ty) {
            assert!(self.$stack > self.$n, "out of list space");
            self.$stack -= 1;
            self.$list[self.$stack] = item;
        }

        /// Pops the items pushed since the stack was at `base`, returning a
        /// span of them in the order they were pushed. An existing span is used
        /// if it has the same items.
        const fn $pop(&mut self, base: usize) -> Span {
            // The stack grows down, so pushed[k] is `list[base - 1 - k]`.
            let len = base - self.$stack;
            // The span starts at the first place the items occur, or the first
            // place a prefix of them ends the list, with the rest appended.
            // That place is still the first occurrence after later appends,
            // so equal lists always get equal spans.
            let mut start = 0;
            loop {
                let mut k = 0;
                while k < len
                    && start + k < self.$n
                    && $eq(&self.$list[start + k], &self.$list[base - 1 - k])
                {
                    k += 1;
                }
                if k == len || start + k == self.$n {
                    break;
                }
                start += 1;
            }
            if start + len > self.$n {
                // `stack == base - len`, so this can't overwrite the stack
                // before it's read.
                assert!(start + len <= self.$stack, "out of list space");
                let mut k = self.$n - start;
                while k < len {
                    self.$list[start + k] = self.$list[base - 1 - k];
                    k += 1;
                }
                self.$n = start + len;
            }
            self.$stack = base;
            Span {
                start: to_u32(start),
                len: to_u32(len),
            }
        }
    };
}

impl Writer<'_> {
    list_ops!(
        push_child,
        pop_children,
        children,
        n_children,
        c_stack,
        u32,
        u32_eq
    );
    list_ops!(
        push_variant,
        pop_variants,
        variants,
        n_variants,
        v_stack,
        TableVariant,
        variants_eq
    );
    list_ops!(push_field, pop_fields, fields, n_fields, f_stack, TableField, fields_eq);

    /// Returns the index of `node`, adding it if it isn't already present.
    const fn node(&mut self, node: TableNode) -> u32 {
        let mut i = 0;
        while i < self.n_nodes {
            if nodes_eq(&node, &self.nodes[i]) {
                return to_u32(i);
            }
            i += 1;
        }
        assert!(self.n_nodes < self.nodes.len(), "out of node space");
        self.nodes[self.n_nodes] = node;
        self.n_nodes += 1;
        to_u32(self.n_nodes - 1)
    }

    /// Returns a reference to `name`, adding it if it isn't already present.
    const fn name(&mut self, name: &str) -> NameRef {
        self.names.intern(name)
    }

    /// Writes a type and the types it contains, returning its node index.
    const fn ty(&mut self, ty: &DataModelType) -> u32 {
        let node = match ty {
            DataModelType::Bool => TableNode::Bool,
            DataModelType::I8 => TableNode::I8,
            DataModelType::U8 => TableNode::U8,
            DataModelType::I16 => TableNode::I16,
            DataModelType::I32 => TableNode::I32,
            DataModelType::I64 => TableNode::I64,
            DataModelType::I128 => TableNode::I128,
            DataModelType::U16 => TableNode::U16,
            DataModelType::U32 => TableNode::U32,
            DataModelType::U64 => TableNode::U64,
            DataModelType::U128 => TableNode::U128,
            DataModelType::F32 => TableNode::F32,
            DataModelType::F64 => TableNode::F64,
            DataModelType::Char => TableNode::Char,
            DataModelType::String { max_len } => TableNode::String {
                max_len: to_u64_opt(*max_len),
            },
            DataModelType::ByteArray { max_len } => TableNode::ByteArray {
                max_len: to_u64_opt(*max_len),
            },
            DataModelType::Option(inner) => TableNode::Option(self.ty(inner)),
            DataModelType::Unit => TableNode::Unit,
            DataModelType::Seq { element, max_len } => TableNode::Seq {
                element: self.ty(element),
                max_len: to_u64_opt(*max_len),
            },
            DataModelType::Tuple(tys) => self.tuple(tys),
            DataModelType::Map { key, val, max_len } => TableNode::Map {
                key: self.ty(key),
                val: self.ty(val),
                max_len: to_u64_opt(*max_len),
            },
            DataModelType::Struct { name, data } => TableNode::Struct {
                name: self.name(name),
                data: self.data(data),
            },
            DataModelType::Enum { name, variants } => {
                let name = self.name(name);
                let base = self.v_stack;
                let mut i = 0;
                while i < variants.len() {
                    let v = self.variant(variants[i]);
                    self.push_variant(v);
                    i += 1;
                }
                TableNode::Enum {
                    name,
                    variants: self.pop_variants(base),
                }
            }
        };
        self.node(node)
    }

    /// A tuple, or an array if its elements are all the same node
    const fn tuple(&mut self, tys: &[&DataModelType]) -> TableNode {
        let base = self.c_stack;
        let mut i = 0;
        while i < tys.len() {
            let idx = self.ty(tys[i]);
            self.push_child(idx);
            i += 1;
        }
        // The stack is in reverse order, which doesn't matter to `array_of`.
        let (_, rest) = self.children.split_at(self.c_stack);
        let (pushed, _) = rest.split_at(base - self.c_stack);
        if let Some((element, len)) = array_of(pushed) {
            self.c_stack = base;
            return TableNode::Array { element, len };
        }
        TableNode::Tuple(self.pop_children(base))
    }

    const fn tys(&mut self, tys: &[&DataModelType]) -> Span {
        let base = self.c_stack;
        let mut i = 0;
        while i < tys.len() {
            let idx = self.ty(tys[i]);
            self.push_child(idx);
            i += 1;
        }
        self.pop_children(base)
    }

    const fn variant(&mut self, var: &Variant) -> TableVariant {
        TableVariant {
            name: self.name(var.name),
            data: self.data(&var.data),
        }
    }

    const fn field(&mut self, field: &NamedField) -> TableField {
        TableField {
            name: self.name(field.name),
            ty: self.ty(field.ty),
        }
    }

    const fn data(&mut self, data: &Data) -> TableData {
        match data {
            Data::Unit => TableData::Unit,
            Data::Newtype(inner) => TableData::Newtype(self.ty(inner)),
            Data::Tuple(tys) => TableData::Tuple(self.tys(tys)),
            Data::Struct(fields) => {
                let base = self.f_stack;
                let mut i = 0;
                while i < fields.len() {
                    let f = self.field(fields[i]);
                    self.push_field(f);
                    i += 1;
                }
                TableData::Struct(self.pop_fields(base))
            }
        }
    }
}

const EMPTY_NAME: NameRef = NameRef { offset: 0, len: 0 };
const EMPTY_VARIANT: TableVariant = TableVariant {
    name: EMPTY_NAME,
    data: TableData::Unit,
};
const EMPTY_FIELD: TableField = TableField {
    name: EMPTY_NAME,
    ty: 0,
};

/// A deduplicated table, built into scratch space: step two of building a
/// table.
///
/// The const parameters are the scratch space, from [`scratch_lengths()`]:
/// roots, nodes, children, variants, fields and name bytes. Only the start of
/// each array is used; see [`TableScratch::lengths()`]. A scratch is not meant
/// to be kept; copy it into a [`SchemaTable`].
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableScratch<
    const ROOTS: usize,
    const NODES: usize,
    const CHILDREN: usize,
    const VARIANTS: usize,
    const FIELDS: usize,
    const NAME_BYTES: usize,
> {
    roots: [u32; ROOTS],
    n_roots: usize,
    // The node index of each input, 1:1 with the inputs given (so `ROOTS` long).
    inputs: [u32; ROOTS],
    nodes: [TableNode; NODES],
    n_nodes: usize,
    children: [u32; CHILDREN],
    n_children: usize,
    variants: [TableVariant; VARIANTS],
    n_variants: usize,
    fields: [TableField; FIELDS],
    n_fields: usize,
    names: [u8; NAME_BYTES],
    n_names: usize,
}

impl<
        const ROOTS: usize,
        const NODES: usize,
        const CHILDREN: usize,
        const VARIANTS: usize,
        const FIELDS: usize,
        const NAME_BYTES: usize,
    > TableScratch<ROOTS, NODES, CHILDREN, VARIANTS, FIELDS, NAME_BYTES>
{
    /// Flattens and deduplicates `roots` into scratch space.
    ///
    /// `STATES` and `EDGES` are the space for interning names, which depends
    /// on `NAME_BYTES`. It's only used while building, and isn't kept. Pass
    /// [`TableLengths::interner_states()`] and
    /// [`TableLengths::interner_edges()`] of the [`scratch_lengths()`] result.
    ///
    /// Fails (at compile time, in const context) if `roots` isn't `ROOTS` long,
    /// if the other const parameters are smaller than [`scratch_lengths()`]
    /// gives, or if `STATES` and `EDGES` are too small for `NAME_BYTES`. The only
    /// valid inputs are the results of the previous step, for the same `roots`.
    ///
    /// The time taken grows with the square of the table's size, and deeply
    /// nested types can exceed rustc's limits; see [`schema_table!`] for the
    /// `long_running_const_eval` and `recursion_limit` notes.
    ///
    /// [`schema_table!`]: crate::schema_table
    pub const fn new<const STATES: usize, const EDGES: usize>(roots: &[&DataModelType]) -> Self {
        assert!(roots.len() == ROOTS, "wrong root count");
        let mut states = [EMPTY_STATE; STATES];
        let mut edges = [EMPTY_EDGE; EDGES];
        let mut out = Self {
            roots: [0; ROOTS],
            n_roots: 0,
            inputs: [0; ROOTS],
            nodes: [TableNode::Unit; NODES],
            n_nodes: 0,
            children: [0; CHILDREN],
            n_children: 0,
            variants: [EMPTY_VARIANT; VARIANTS],
            n_variants: 0,
            fields: [EMPTY_FIELD; FIELDS],
            n_fields: 0,
            names: [0; NAME_BYTES],
            n_names: 0,
        };
        let mut w = Writer {
            nodes: &mut out.nodes,
            n_nodes: 0,
            children: &mut out.children,
            n_children: 0,
            c_stack: CHILDREN,
            variants: &mut out.variants,
            n_variants: 0,
            v_stack: VARIANTS,
            fields: &mut out.fields,
            n_fields: 0,
            f_stack: FIELDS,
            names: Interner::new(&mut out.names, &mut states, &mut edges),
        };
        // Each distinct type is a root once, in the order first given.
        // Searching the roots so far is quadratic.
        let mut idx = [0; ROOTS];
        let mut inputs = [0; ROOTS];
        let mut n_roots = 0;
        let mut i = 0;
        while i < ROOTS {
            let node = w.ty(roots[i]);
            inputs[i] = node;
            let mut k = 0;
            while k < n_roots && idx[k] != node {
                k += 1;
            }
            if k == n_roots {
                idx[n_roots] = node;
                n_roots += 1;
            }
            i += 1;
        }
        out.roots = idx;
        out.inputs = inputs;
        out.n_roots = n_roots;
        out.n_nodes = w.n_nodes;
        out.n_children = w.n_children;
        out.n_variants = w.n_variants;
        out.n_fields = w.n_fields;
        out.n_names = w.names.n_names;
        out
    }

    /// The exact array lengths needed to hold this table, to be used as the
    /// const parameters of [`SchemaTable`]: step three.
    pub const fn lengths(&self) -> TableLengths {
        TableLengths {
            roots: self.n_roots,
            nodes: self.n_nodes,
            children: self.n_children,
            variants: self.n_variants,
            fields: self.n_fields,
            names: self.n_names,
        }
    }

    /// The node index of each of the `roots` given to [`TableScratch::new()`],
    /// in the order given, including repeats.
    ///
    /// Fails (at compile time, in const context) if `INPUTS` isn't `ROOTS`.
    pub const fn inputs<const INPUTS: usize>(&self) -> [u32; INPUTS] {
        assert!(INPUTS == ROOTS, "wrong input count");
        let mut out = [0; INPUTS];
        let mut i = 0;
        while i < INPUTS {
            out[i] = self.inputs[i];
            i += 1;
        }
        out
    }
}

// ---- Step three: right-sized arrays ----

/// A table of one or more schemas, built at compile time
///
/// The const parameters are the numbers of roots, nodes, children, variants,
/// fields and name bytes. Use [`schema_table!`] rather than building this
/// directly; the type only needs writing out to keep a table in a
/// `static`.
///
/// [`impl_schema_table!`] builds a table whose type can be named, with
/// [`schema_table_type!`].
///
/// The fields are public, so a table can be built by hand. Reading, serializing
/// or converting a table panics if its spans, names or indices are out of
/// bounds, which can't happen for one built by this crate.
///
/// [`schema_table!`]: crate::schema_table
/// [`impl_schema_table!`]: crate::impl_schema_table
/// [`schema_table_type!`]: crate::schema_table_type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaTable<
    const ROOTS: usize,
    const NODES: usize,
    const CHILDREN: usize,
    const VARIANTS: usize,
    const FIELDS: usize,
    const NAME_BYTES: usize,
> {
    /// Always [`VERSION`]
    pub version: u32,
    /// The node index of each distinct type, in the order they were given
    pub roots: [u32; ROOTS],
    /// Unique types, each after any of its contained types
    pub nodes: [TableNode; NODES],
    /// The elements of tuples: node indices, referred to by [`Span`]s
    pub children: [u32; CHILDREN],
    /// The variants of enums, referred to by [`Span`]s
    pub variants: [TableVariant; VARIANTS],
    /// The fields of structs and variants, referred to by [`Span`]s
    pub fields: [TableField; FIELDS],
    /// The bytes of every name, referred to by [`NameRef`]s. These are always
    /// UTF-8, and every [`NameRef`] is on char boundaries.
    pub names: [u8; NAME_BYTES],
}

/// Copies the first `M` items of `from` into an array (in const context)
macro_rules! copy_prefix {
    ($from:expr, $empty:expr, $m:expr) => {{
        let mut out = [$empty; $m];
        let mut i = 0;
        while i < $m {
            out[i] = $from[i];
            i += 1;
        }
        out
    }};
}

impl<
        const ROOTS: usize,
        const NODES: usize,
        const CHILDREN: usize,
        const VARIANTS: usize,
        const FIELDS: usize,
        const NAME_BYTES: usize,
    > SchemaTable<ROOTS, NODES, CHILDREN, VARIANTS, FIELDS, NAME_BYTES>
{
    /// Copies a table out of scratch space: step three of building a table.
    ///
    /// Fails (at compile time, in const context) if the const parameters don't
    /// match [`TableScratch::lengths()`] of `scratch`. The only valid inputs
    /// are the lengths of the scratch passed in.
    #[doc(hidden)]
    pub const fn new<
        const SCRATCH_ROOTS: usize,
        const SCRATCH_NODES: usize,
        const SCRATCH_CHILDREN: usize,
        const SCRATCH_VARIANTS: usize,
        const SCRATCH_FIELDS: usize,
        const SCRATCH_NAME_BYTES: usize,
    >(
        scratch: &TableScratch<
            SCRATCH_ROOTS,
            SCRATCH_NODES,
            SCRATCH_CHILDREN,
            SCRATCH_VARIANTS,
            SCRATCH_FIELDS,
            SCRATCH_NAME_BYTES,
        >,
    ) -> Self {
        assert!(scratch.n_roots == ROOTS, "wrong root count");
        assert!(scratch.n_nodes == NODES, "wrong node count");
        assert!(scratch.n_children == CHILDREN, "wrong children count");
        assert!(scratch.n_variants == VARIANTS, "wrong variant count");
        assert!(scratch.n_fields == FIELDS, "wrong field count");
        assert!(scratch.n_names == NAME_BYTES, "wrong name byte count");
        Self {
            version: VERSION,
            roots: copy_prefix!(scratch.roots, 0, ROOTS),
            nodes: copy_prefix!(scratch.nodes, TableNode::Unit, NODES),
            children: copy_prefix!(scratch.children, 0, CHILDREN),
            variants: copy_prefix!(scratch.variants, EMPTY_VARIANT, VARIANTS),
            fields: copy_prefix!(scratch.fields, EMPTY_FIELD, FIELDS),
            names: copy_prefix!(scratch.names, 0, NAME_BYTES),
        }
    }

    /// A borrowed view of this table, which is how it's serialized
    pub const fn as_table_ref(&self) -> SchemaTableRef<'_> {
        let names = match core::str::from_utf8(&self.names) {
            Ok(s) => s,
            Err(_) => panic!("names are not UTF-8"),
        };
        SchemaTableRef {
            version: self.version,
            roots: &self.roots,
            nodes: &self.nodes,
            children: &self.children,
            variants: &self.variants,
            fields: &self.fields,
            names,
        }
    }
}

impl<
        const ROOTS: usize,
        const NODES: usize,
        const CHILDREN: usize,
        const VARIANTS: usize,
        const FIELDS: usize,
        const NAME_BYTES: usize,
    > Serialize for SchemaTable<ROOTS, NODES, CHILDREN, VARIANTS, FIELDS, NAME_BYTES>
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_table_ref().serialize(serializer)
    }
}

/// A borrowed [`SchemaTable`], of any size
///
/// A [`SchemaTable`] serializes as this. Its fields are as in [`SchemaTable`],
/// with `names` as a `str`.
///
/// As with [`SchemaTable`], reading, serializing or converting one built by
/// hand panics if it's out of bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct SchemaTableRef<'a> {
    pub version: u32,
    pub roots: &'a [u32],
    pub nodes: &'a [TableNode],
    pub children: &'a [u32],
    pub variants: &'a [TableVariant],
    pub fields: &'a [TableField],
    pub names: &'a str,
}

impl<'a> SchemaTableRef<'a> {
    /// The name that `name` refers to
    ///
    /// Panics if `name` is out of bounds or not on char boundaries, which
    /// can't happen for a reference from a table built by this crate.
    pub const fn name(&self, name: NameRef) -> &'a str {
        let (_, rest) = self.names.split_at(name.offset as usize);
        rest.split_at(name.len as usize).0
    }

    /// The node indices of a span of [`SchemaTableRef::children`]
    ///
    /// Panics if `span` is out of bounds, which can't happen for a span from a
    /// table built by this crate.
    pub const fn children_of(&self, span: Span) -> &'a [u32] {
        let (_, rest) = self.children.split_at(span.start as usize);
        rest.split_at(span.len as usize).0
    }

    /// The variants of a span of [`SchemaTableRef::variants`]
    ///
    /// Panics if `span` is out of bounds, which can't happen for a span from a
    /// table built by this crate.
    pub const fn variants_of(&self, span: Span) -> &'a [TableVariant] {
        let (_, rest) = self.variants.split_at(span.start as usize);
        rest.split_at(span.len as usize).0
    }

    /// The fields of a span of [`SchemaTableRef::fields`]
    ///
    /// Panics if `span` is out of bounds, which can't happen for a span from a
    /// table built by this crate.
    pub const fn fields_of(&self, span: Span) -> &'a [TableField] {
        let (_, rest) = self.fields.split_at(span.start as usize);
        rest.split_at(span.len as usize).0
    }
}

// ---- Serializing, with each node's lists inside it ----

/// The serde names of the encoding's types, which the schemas in `schemas.rs`
/// and the owned types use too.
pub(crate) mod names {
    pub const TABLE: &str = "SchemaTable";
    pub const NODE: &str = "TableNode";
    pub const DATA: &str = "TableData";
    pub const VARIANT: &str = "TableVariant";
}

// schema[impl table.encoding]
impl Serialize for SchemaTableRef<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct(names::TABLE, 4)?;
        s.serialize_field("version", &self.version)?;
        s.serialize_field("roots", self.roots)?;
        s.serialize_field("nodes", &Nodes(self))?;
        s.serialize_field("names", self.names)?;
        s.end()
    }
}

struct Nodes<'a, 'b>(&'b SchemaTableRef<'a>);

impl Serialize for Nodes<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.nodes.iter().map(|node| NodeSer(self.0, node)))
    }
}

struct NodeSer<'a, 'b>(&'b SchemaTableRef<'a>, &'b TableNode);

impl Serialize for NodeSer<'_, '_> {
    fn serialize<Z: Serializer>(&self, s: Z) -> Result<Z::Ok, Z::Error> {
        use names::NODE;
        use TableNode as T;
        let t = self.0;
        let unit = |s: Z, idx, name| s.serialize_unit_variant(NODE, idx, name);
        match *self.1 {
            T::Bool => unit(s, 0x00, "Bool"),
            T::I8 => unit(s, 0x01, "I8"),
            T::U8 => unit(s, 0x02, "U8"),
            T::I16 => unit(s, 0x03, "I16"),
            T::I32 => unit(s, 0x04, "I32"),
            T::I64 => unit(s, 0x05, "I64"),
            T::I128 => unit(s, 0x06, "I128"),
            T::U16 => unit(s, 0x07, "U16"),
            T::U32 => unit(s, 0x08, "U32"),
            T::U64 => unit(s, 0x09, "U64"),
            T::U128 => unit(s, 0x0A, "U128"),
            T::F32 => unit(s, 0x0B, "F32"),
            T::F64 => unit(s, 0x0C, "F64"),
            T::Char => unit(s, 0x0D, "Char"),
            T::String { max_len } => {
                let mut v = s.serialize_struct_variant(NODE, 0x0E, "String", 1)?;
                v.serialize_field("max_len", &max_len)?;
                v.end()
            }
            T::ByteArray { max_len } => {
                let mut v = s.serialize_struct_variant(NODE, 0x0F, "ByteArray", 1)?;
                v.serialize_field("max_len", &max_len)?;
                v.end()
            }
            T::Option(inner) => s.serialize_newtype_variant(NODE, 0x10, "Option", &inner),
            T::Unit => unit(s, 0x11, "Unit"),
            T::Seq { element, max_len } => {
                let mut v = s.serialize_struct_variant(NODE, 0x12, "Seq", 2)?;
                v.serialize_field("element", &element)?;
                v.serialize_field("max_len", &max_len)?;
                v.end()
            }
            T::Tuple(span) => s.serialize_newtype_variant(NODE, 0x13, "Tuple", t.children_of(span)),
            T::Map { key, val, max_len } => {
                let mut v = s.serialize_struct_variant(NODE, 0x14, "Map", 3)?;
                v.serialize_field("key", &key)?;
                v.serialize_field("val", &val)?;
                v.serialize_field("max_len", &max_len)?;
                v.end()
            }
            T::Struct { name, data } => {
                let mut v = s.serialize_struct_variant(NODE, 0x15, "Struct", 2)?;
                v.serialize_field("name", &name)?;
                v.serialize_field("data", &DataSer(t, data))?;
                v.end()
            }
            T::Enum { name, variants } => {
                let mut v = s.serialize_struct_variant(NODE, 0x16, "Enum", 2)?;
                v.serialize_field("name", &name)?;
                v.serialize_field("variants", &Variants(t, variants))?;
                v.end()
            }
            T::Array { element, len } => {
                let mut v = s.serialize_struct_variant(NODE, 0x17, "Array", 2)?;
                v.serialize_field("element", &element)?;
                v.serialize_field("len", &len)?;
                v.end()
            }
        }
    }
}

struct DataSer<'a, 'b>(&'b SchemaTableRef<'a>, TableData);

impl Serialize for DataSer<'_, '_> {
    fn serialize<Z: Serializer>(&self, s: Z) -> Result<Z::Ok, Z::Error> {
        use names::DATA;
        match self.1 {
            TableData::Unit => s.serialize_unit_variant(DATA, 0, "Unit"),
            TableData::Newtype(n) => s.serialize_newtype_variant(DATA, 1, "Newtype", &n),
            TableData::Tuple(span) => {
                s.serialize_newtype_variant(DATA, 2, "Tuple", self.0.children_of(span))
            }
            TableData::Struct(span) => {
                s.serialize_newtype_variant(DATA, 3, "Struct", self.0.fields_of(span))
            }
        }
    }
}

struct Variants<'a, 'b>(&'b SchemaTableRef<'a>, Span);

impl Serialize for Variants<'_, '_> {
    fn serialize<Z: Serializer>(&self, s: Z) -> Result<Z::Ok, Z::Error> {
        let t = self.0;
        s.collect_seq(t.variants_of(self.1).iter().map(|v| VariantSer(t, v)))
    }
}

struct VariantSer<'a, 'b>(&'b SchemaTableRef<'a>, &'b TableVariant);

impl Serialize for VariantSer<'_, '_> {
    fn serialize<Z: Serializer>(&self, s: Z) -> Result<Z::Ok, Z::Error> {
        let mut v = s.serialize_struct(names::VARIANT, 2)?;
        v.serialize_field("name", &self.1.name)?;
        v.serialize_field("data", &DataSer(self.0, self.1.data))?;
        v.end()
    }
}

/// Builds a [`SchemaTable`] of the schemas of one or more types, in const
/// context.
///
/// Each distinct type is a root once, in the order the types were first
/// given. The types can't use the generic parameters of an enclosing item, as
/// the table is built in inner `const` items.
///
/// ```
/// use postcard_schema::{schema_table, table::node::TableNode};
///
/// let table = schema_table!(u8, Option<u8>);
/// assert_eq!(table.nodes, [TableNode::U8, TableNode::Option(0)]);
/// assert_eq!(table.roots, [0, 1]);
/// ```
///
/// Instead of types, `schemas = EXPR` takes a const expression of type
/// `&[&DataModelType]`. It must be const-evaluable and can't use the generic
/// parameters of an enclosing item.
///
/// ```
/// use postcard_schema::{schema::DataModelType, schema_table, Schema};
///
/// const A: &DataModelType = <u8 as Schema>::SCHEMA;
/// const S: &[&DataModelType] = &[A, <Option<u8> as Schema>::SCHEMA];
/// assert_eq!(schema_table!(schemas = S), schema_table!(u8, Option<u8>));
/// ```
///
/// A large table can exceed rustc's limit on const evaluation, which fails with
/// "constant evaluation is taking a long time". Allowing
/// `long_running_const_eval` on the statement or item that uses the macro
/// lifts the limit:
///
/// ```
/// use postcard_schema::schema_table;
///
/// #[allow(long_running_const_eval)]
/// let table = schema_table!(u8, Option<u8>);
/// # assert_eq!(table.roots, [0, 1]);
/// ```
///
/// If you require this `allow`, please open an issue! There are potential
/// optimizations possible, but they have been avoided for now as they make the
/// (already complex) const-fn code more complex.
///
/// The time taken grows with the square of the table's size, and rustc keeps
/// warning that evaluation is taking a long time, even with the `allow`.
///
/// Types nested more than about 40 levels deep exceed rustc's limit on const
/// evaluation's stack, which fails with "reached the configured maximum
/// number of stack frames". Raising `#![recursion_limit]` in the crate that
/// uses the macro raises that limit too.
///
/// The table isn't checked against a [`TableLimits`], so a peer decoding it
/// with the default limits rejects types nested more than 128 deep, or with
/// more than 65536 types, variants and fields.
///
/// A large table is best kept in a `const` or `static`, as a `let` copies it
/// onto the stack.
///
/// [`SchemaTable`]: crate::table::SchemaTable
/// [`TableLimits`]: crate::table::TableLimits
#[macro_export]
macro_rules! schema_table {
    (schemas = $schemas:expr $(,)?) => {{
        $crate::__schema_table_consts!(schemas = $schemas);
        __SchemaTable::SCHEMA_TABLE
    }};
    ($($ty:ty),+ $(,)?) => {{
        $crate::__schema_table_consts!($($ty),+);
        __SchemaTable::SCHEMA_TABLE
    }};
}

/// Builds the postcard encoding of a [`SchemaTable`] of the schemas of one or
/// more types, as a byte array, in const context.
///
/// This is the same as serializing [`schema_table!`]'s table with postcard,
/// with no work at runtime. The types can't use the generic parameters of an
/// enclosing item.
///
/// `schemas = EXPR` is accepted as in [`schema_table!`].
///
/// ```
/// use postcard_schema::schema_table_bytes;
///
/// static SCHEMAS: &[u8] = &schema_table_bytes!(u8, Option<u8>);
/// assert_eq!(SCHEMAS, [1, 2, 0, 1, 2, 2, 0x10, 0, 0]);
/// ```
///
/// As with [`schema_table!`], `long_running_const_eval` can be allowed for a
/// large table, and `#![recursion_limit]` raised for deeply nested types:
///
/// ```
/// use postcard_schema::schema_table_bytes;
///
/// #[allow(long_running_const_eval)]
/// static SCHEMAS: &[u8] = &schema_table_bytes!(u8, Option<u8>);
/// # assert_eq!(SCHEMAS.len(), 9);
/// ```
///
/// [`SchemaTable`]: crate::table::SchemaTable
/// [`schema_table!`]: crate::schema_table
#[macro_export]
macro_rules! schema_table_bytes {
    (schemas = $schemas:expr $(,)?) => {{
        $crate::__schema_table_consts!(schemas = $schemas);
        const __BYTES: [u8; __SchemaTable::SCHEMA_TABLE.encoded_len()] =
            __SchemaTable::SCHEMA_TABLE.to_bytes();
        __BYTES
    }};
    ($($ty:ty),+ $(,)?) => {{
        $crate::__schema_table_consts!($($ty),+);
        const __BYTES: [u8; __SchemaTable::SCHEMA_TABLE.encoded_len()] =
            __SchemaTable::SCHEMA_TABLE.to_bytes();
        __BYTES
    }};
}

/// Builds a [`SchemaTable`] into associated consts of a type, in const context,
/// so that the table's type can be named with [`schema_table_type!`].
///
/// `impl_schema_table!(Type, A, B, ..)` adds an `impl Type` block with:
///
/// * `SCHEMA_TABLE`: the table of the schemas of `A, B, ..`, as built by
///   [`schema_table!`]. Its type is `schema_table_type!(Type)`.
/// * `SCHEMA_INPUTS`: a `[u32; N]` of the node index of each of the `N` schemas
///   given, in order, including repeats and schemas nested in other inputs.
///
/// `schemas = EXPR` is accepted as in [`schema_table!`], and the same notes on
/// rustc's limits apply. A visibility before `Type`, such as
/// `impl_schema_table!(pub Type, ..)`, is given to the consts. The block also
/// holds hidden consts named `__PS_*`, so a type can only have one table, and
/// its other associated items shouldn't use those names.
///
/// ```
/// use postcard_schema::{impl_schema_table, schema_table_type, table::node::TableNode};
///
/// struct Helper;
/// impl_schema_table!(Helper, Option<u8>, u8, Option<u8>);
///
/// static TABLE: schema_table_type!(Helper) = Helper::SCHEMA_TABLE;
/// assert_eq!(TABLE.nodes, [TableNode::U8, TableNode::Option(0)]);
/// assert_eq!(TABLE.roots, [1, 0]);
/// assert_eq!(Helper::SCHEMA_INPUTS, [1, 0, 1]);
/// ```
///
/// The consts can refer to the type's other associated consts, which
/// can in turn use the table, so a macro can build a type holding a table:
///
/// ```
/// use postcard_schema::{
///     impl_schema_table, schema::DataModelType, schema_table_type, Schema,
/// };
///
/// struct Helper;
/// impl Helper {
///     const SCHEMAS: [&'static DataModelType; 2] =
///         [<u8 as Schema>::SCHEMA, <bool as Schema>::SCHEMA];
///     const PAIR: (schema_table_type!(Helper), [u32; 2]) =
///         (Helper::SCHEMA_TABLE, Helper::SCHEMA_INPUTS);
/// }
/// impl_schema_table!(Helper, schemas = &Helper::SCHEMAS);
///
/// assert_eq!(Helper::PAIR.1, [0, 1]);
/// ```
///
/// [`SchemaTable`]: crate::table::SchemaTable
/// [`schema_table!`]: crate::schema_table
/// [`schema_table_type!`]: crate::schema_table_type
#[macro_export]
macro_rules! impl_schema_table {
    // Each step is its own const, as each const generic argument is evaluated
    // separately; naming a const evaluates it once. The consts use `$ty::`
    // rather than `Self::`, which isn't allowed in const arguments.
    ($vis:vis $ty:ident, schemas = $schemas:expr $(,)?) => {
        #[allow(dead_code)]
        impl $ty {
            #[doc(hidden)]
            $vis const __PS_ROOTS: &'static [&'static $crate::schema::DataModelType] = $schemas;
            #[doc(hidden)]
            $vis const __PS_MAX: $crate::table::TableLengths =
                $crate::table::scratch_lengths($ty::__PS_ROOTS);
            #[doc(hidden)]
            $vis const __PS_SCRATCH: $crate::table::TableScratch<
                { $ty::__PS_MAX.roots },
                { $ty::__PS_MAX.nodes },
                { $ty::__PS_MAX.children },
                { $ty::__PS_MAX.variants },
                { $ty::__PS_MAX.fields },
                { $ty::__PS_MAX.names },
            > = $crate::table::TableScratch::new::<
                { $ty::__PS_MAX.interner_states() },
                { $ty::__PS_MAX.interner_edges() },
            >($ty::__PS_ROOTS);
            #[doc(hidden)]
            $vis const __PS_LEN: $crate::table::TableLengths = $ty::__PS_SCRATCH.lengths();
            /// The table of the schemas given to `impl_schema_table!`
            $vis const SCHEMA_TABLE: $crate::schema_table_type!($ty) =
                $crate::table::SchemaTable::new(&$ty::__PS_SCRATCH);
            /// The node index of each schema given to `impl_schema_table!`
            $vis const SCHEMA_INPUTS: [u32; $ty::__PS_ROOTS.len()] =
                $ty::__PS_SCRATCH.inputs();
        }
    };
    ($vis:vis $ty:ident, $($t:ty),+ $(,)?) => {
        $crate::impl_schema_table!(
            $vis $ty, schemas = &[$(<$t as $crate::Schema>::SCHEMA),+]
        );
    };
}

/// The type of `Type::SCHEMA_TABLE`, built by [`impl_schema_table!`]
///
/// This is a [`SchemaTable`] with its const parameters filled in, and can be
/// used wherever a type can, including as a generic argument. `Type` can be a
/// path, such as `module::Type`, if the consts are visible there.
///
/// [`SchemaTable`]: crate::table::SchemaTable
/// [`impl_schema_table!`]: crate::impl_schema_table
#[macro_export]
macro_rules! schema_table_type {
    ($($seg:ident)::+) => {
        $crate::table::SchemaTable<
            { $($seg)::+::__PS_LEN.roots },
            { $($seg)::+::__PS_LEN.nodes },
            { $($seg)::+::__PS_LEN.children },
            { $($seg)::+::__PS_LEN.variants },
            { $($seg)::+::__PS_LEN.fields },
            { $($seg)::+::__PS_LEN.names },
        >
    };
}

/// The const items that build a table, shared by [`schema_table!`] and
/// [`schema_table_bytes!`].
///
/// The table is `__SchemaTable::SCHEMA_TABLE`.
#[doc(hidden)]
#[macro_export]
macro_rules! __schema_table_consts {
    (schemas = $schemas:expr) => {
        struct __SchemaTable;
        $crate::impl_schema_table!(__SchemaTable, schemas = $schemas);
    };
    ($($ty:ty),+) => {
        $crate::__schema_table_consts!(
            schemas = &[$(<$ty as $crate::Schema>::SCHEMA),+]
        );
    };
}

// ---- const equality, as trait impls can't be const yet ----
//
// Equal lists always get equal spans (see `list_ops!`), so lists are compared
// by their spans.

const fn u32_eq(a: &u32, b: &u32) -> bool {
    *a == *b
}

const fn names_eq(a: &NameRef, b: &NameRef) -> bool {
    a.offset == b.offset && a.len == b.len
}

const fn spans_eq(a: &Span, b: &Span) -> bool {
    a.start == b.start && a.len == b.len
}

const fn max_eq(a: &Option<u64>, b: &Option<u64>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => *a == *b,
        _ => false,
    }
}

const fn data_eq(a: &TableData, b: &TableData) -> bool {
    match (a, b) {
        (TableData::Unit, TableData::Unit) => true,
        (TableData::Newtype(a), TableData::Newtype(b)) => *a == *b,
        (TableData::Tuple(a), TableData::Tuple(b)) => spans_eq(a, b),
        (TableData::Struct(a), TableData::Struct(b)) => spans_eq(a, b),
        _ => false,
    }
}

const fn variants_eq(a: &TableVariant, b: &TableVariant) -> bool {
    names_eq(&a.name, &b.name) && data_eq(&a.data, &b.data)
}

const fn fields_eq(a: &TableField, b: &TableField) -> bool {
    names_eq(&a.name, &b.name) && a.ty == b.ty
}

const fn nodes_eq(a: &TableNode, b: &TableNode) -> bool {
    use TableNode as T;
    match (a, b) {
        (T::Bool, T::Bool)
        | (T::I8, T::I8)
        | (T::U8, T::U8)
        | (T::I16, T::I16)
        | (T::I32, T::I32)
        | (T::I64, T::I64)
        | (T::I128, T::I128)
        | (T::U16, T::U16)
        | (T::U32, T::U32)
        | (T::U64, T::U64)
        | (T::U128, T::U128)
        | (T::F32, T::F32)
        | (T::F64, T::F64)
        | (T::Char, T::Char)
        | (T::Unit, T::Unit) => true,
        (T::String { max_len: a }, T::String { max_len: b })
        | (T::ByteArray { max_len: a }, T::ByteArray { max_len: b }) => max_eq(a, b),
        (T::Option(a), T::Option(b)) => *a == *b,
        (
            T::Seq {
                element: ea,
                max_len: ma,
            },
            T::Seq {
                element: eb,
                max_len: mb,
            },
        ) => *ea == *eb && max_eq(ma, mb),
        (T::Tuple(a), T::Tuple(b)) => spans_eq(a, b),
        (
            T::Map {
                key: ka,
                val: va,
                max_len: ma,
            },
            T::Map {
                key: kb,
                val: vb,
                max_len: mb,
            },
        ) => *ka == *kb && *va == *vb && max_eq(ma, mb),
        (T::Struct { name: na, data: da }, T::Struct { name: nb, data: db }) => {
            names_eq(na, nb) && data_eq(da, db)
        }
        (
            T::Enum {
                name: na,
                variants: va,
            },
            T::Enum {
                name: nb,
                variants: vb,
            },
        ) => names_eq(na, nb) && spans_eq(va, vb),
        (
            T::Array {
                element: ea,
                len: la,
            },
            T::Array {
                element: eb,
                len: lb,
            },
        ) => *ea == *eb && *la == *lb,
        _ => false,
    }
}

/// The element and length of a tuple of these node indices, if it's written as
/// a [`TableNode::Array`]: two or more elements, all the same node
// schema[impl table.array]
const fn array_of(idxs: &[u32]) -> Option<(u32, u64)> {
    if idxs.len() < 2 {
        return None;
    }
    let first = idxs[0];
    let mut i = 1;
    while i < idxs.len() {
        if idxs[i] != first {
            return None;
        }
        i += 1;
    }
    Some((first, idxs.len() as u64))
}

/// `usize` to `u32`, failing (at compile time) if it doesn't fit
///
/// `u32::MAX` itself is rejected, as the interner uses it to mean "none".
const fn to_u32(n: usize) -> u32 {
    assert!(n < u32::MAX as usize, "too large for a schema table");
    n as u32
}

const fn to_u64_opt(n: Option<usize>) -> Option<u64> {
    match n {
        Some(n) => Some(n as u64),
        None => None,
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
mod decode;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use decode::{TableError, TableLimits};
mod encode;
mod intern;
#[cfg(any(feature = "std", feature = "alloc"))]
mod owned;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use owned::{OwnedSchemaTable, RawOwnedSchemaTable};
pub mod node;
mod schemas;

#[cfg(all(test, feature = "derive", feature = "std"))]
mod tests;
