//! The parts of a schema table: its nodes, and the lists and names they refer
//! to
//!
//! [`TableNode`] and the types it holds are the nodes of a
//! [`SchemaTable`]. [`OwnedTableNode`] and the types it holds are the nodes of
//! an [`OwnedSchemaTable`], which hold their lists directly.
//!
//! [`SchemaTable`]: super::SchemaTable
//! [`OwnedSchemaTable`]: super::OwnedSchemaTable

use serde::{Deserialize, Serialize};

#[cfg(any(feature = "std", feature = "alloc"))]
pub use super::owned::{OwnedTableData, OwnedTableNode, OwnedTableVariant};

/// A name: a byte range of the table's `names` string
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NameRef {
    /// Offset of the first byte of the name
    pub offset: u32,
    /// Length of the name in bytes
    pub len: u32,
}

/// A field of a struct or struct variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TableField {
    /// The field's name
    pub name: NameRef,
    /// The `nodes` index of the field's type
    pub ty: u32,
}

/// A range of one of a [`SchemaTable`](super::SchemaTable)'s lists: [`SchemaTable::children`](super::SchemaTable::children),
/// [`SchemaTable::variants`](super::SchemaTable::variants) or [`SchemaTable::fields`](super::SchemaTable::fields)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Offset of the first item
    pub start: u32,
    /// Number of items
    pub len: u32,
}

/// The contents of a struct or variant in a [`SchemaTable`](super::SchemaTable)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableData {
    /// [`Data::Unit`](crate::schema::Data::Unit)
    Unit,
    /// [`Data::Newtype`](crate::schema::Data::Newtype), with the node index of its type
    Newtype(u32),
    /// [`Data::Tuple`](crate::schema::Data::Tuple), with a span of [`SchemaTable::children`](super::SchemaTable::children)
    Tuple(Span),
    /// [`Data::Struct`](crate::schema::Data::Struct), with a span of [`SchemaTable::fields`](super::SchemaTable::fields)
    Struct(Span),
}

/// A variant of an enum in a [`SchemaTable`](super::SchemaTable)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TableVariant {
    /// The variant's name
    pub name: NameRef,
    /// The variant's contents
    pub data: TableData,
}

/// One type in a [`SchemaTable`](super::SchemaTable)
///
/// Each `u32` is the index of a node, which appears before this node in the
/// `nodes` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum TableNode {
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
    /// A span of [`SchemaTable::children`](super::SchemaTable::children)
    Tuple(Span),
    Map {
        key: u32,
        val: u32,
        max_len: Option<u64>,
    },
    Struct {
        name: NameRef,
        data: TableData,
    },
    /// A span of [`SchemaTable::variants`](super::SchemaTable::variants)
    Enum {
        name: NameRef,
        variants: Span,
    },
    /// A tuple of `len` copies of `element`, such as an array. Builders write
    /// this for tuples of two or more elements that are all the same node.
    Array {
        element: u32,
        len: u64,
    },
}
