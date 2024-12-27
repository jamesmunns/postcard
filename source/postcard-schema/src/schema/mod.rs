//! ## Schema types
//!
//! The types in this module are used to define the schema of a given data type.
//!
//! The **Postcard Data Model** is nearly identical to the **Serde Data Model**, however Postcard also
//! allows for one additional type, `Schema`, which maps to the [`DataModelType`] type, allowing
//! the schema of types to also be sent over the wire and implement the `Schema` trait.
//!
//! ## Borrowed vs Owned
//!
//! For reasons that have to do with allowing for arbitrarily sized and nested schemas that
//! can be created at compile/const time, as well as being usable in `no-std` contexts, the
//! schema types in this module are implemented using a LOT of `&'static` references.
//!
//! This is useful in those limited contexts, however it makes it difficult to do things
//! like deserialize them, as you can't generally get static references at runtime without
//! a lot of leaking.
//!
//! For cases like this, the [`owned`] module exists, which has copies of all of the "borrowed"
//! versions of the Data Model types. These owned types implement `From` for their borrowed
//! counterpoint, so if you need to deserialize something, you probably want the Owned variant!

#[cfg(any(feature = "use-std", feature = "alloc"))]
pub mod owned;

#[cfg(any(feature = "use-std", feature = "alloc"))]
pub mod fmt;

use serde::Serialize;

/// This enum lists which of the Data Model Types apply to a given type. This describes how the
/// type is encoded on the wire.
///
/// This enum contains all Serde Data Model types as well as a "Schema" Type,
/// which corresponds to [`DataModelType`] itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum DataModelType {
    /// The `bool` Serde Data Model Type
    Bool,

    /// The `i8` Serde Data Model Type
    I8,

    /// The `u8` Serde Data Model Type
    U8,

    /// A variably encoded i16
    I16,

    /// A variably encoded i32
    I32,

    /// A variably encoded i64
    I64,

    /// A variably encoded i128
    I128,

    /// A variably encoded u16
    U16,

    /// A variably encoded u32
    U32,

    /// A variably encoded u64
    U64,

    /// A variably encoded u128
    U128,

    /// A variably encoded usize
    Usize,

    /// A variably encoded isize
    Isize,

    /// The `f32` Serde Data Model Type
    F32,

    /// The `f64` Serde Data Model Type
    F64,

    /// The `char` Serde Data Model Type
    Char,

    /// The `String` Serde Data Model Type
    String,

    /// The `&[u8]` Serde Data Model Type
    ByteArray,

    /// The `Option<T>` Serde Data Model Type
    Option(&'static Self),

    /// The `()` Serde Data Model Type
    Unit,

    /// The "Sequence" Serde Data Model Type
    Seq(&'static Self),

    /// The "Tuple" Serde Data Model Type
    Tuple(&'static [&'static Self]),

    /// The "Map" Serde Data Model Type
    Map {
        /// The map "Key" type
        key: &'static Self,
        /// The map "Value" type
        val: &'static Self,
    },

    /// One of the struct Serde Data Model types
    Struct {
        /// The name of this struct
        name: &'static str,
        /// The data contained in this struct
        data: Data,
    },

    /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
    Enum {
        /// The name of this struct
        name: &'static str,
        /// The variants contained in this enum
        variants: &'static [&'static Variant],
    },

    /// A [`DataModelType`]/[`OwnedDataModelType`](owned::OwnedDataModelType)
    Schema,
}

/// The contents of a struct or enum variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Data {
    /// The "Unit Struct" or "Unit Variant" Serde Data Model Type
    Unit,

    /// The "Newtype Struct" or "Newtype Variant" Serde Data Model Type
    Newtype(&'static DataModelType),

    /// The "Tuple Struct" or "Tuple Variant" Serde Data Model Type
    Tuple(&'static [&'static DataModelType]),

    /// The "Struct" or "Struct Variant" Serde Data Model Type
    Struct(&'static [&'static NamedField]),
}

/// This represents a named struct field.
///
/// For example, in `struct Ex { a: u32 }` the field `a` would be reflected as
/// `NamedField { name: "a", ty: DataModelType::U32 }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedField {
    /// The name of this field
    pub name: &'static str,
    /// The type of this field
    pub ty: &'static DataModelType,
}

/// An enum variant e.g. `T::Bar(...)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Variant {
    /// The name of this variant
    pub name: &'static str,
    /// The data contained in this variant
    pub data: Data,
}
