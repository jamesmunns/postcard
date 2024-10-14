//! ## Schema types
//!
//! The types in this module are used to define the schema of a given data type.
//!
//! The **Postcard Data Model** is nearly identical to the **Serde Data Model**, however Postcard also
//! allows for one additional type, `Schema`, which maps to the [`NamedValue`] type, allowing
//! the schema of types to also be sent over the wire and implement the `Schema` trait.
//!
//! ## Borrowed vs Owned
//!
//! For reasons that have to do with allowing for arbitrarily sized and nexted schemas that
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

use serde::{Deserialize, Serialize};

/// A "NamedType" is used to describe the schema of a given type.
///
/// It contains two pieces of information:
///
/// * A `name`, which is the name of the type, e.g. "u8" for [`u8`].
/// * A `ty`, which is one of the possible [`DataModelType`]s any given type can be represented as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedType {
    /// The name of this type
    pub name: &'static str,
    /// The type
    pub ty: &'static DataModelType,
}

/// This enum lists which of the Data Model Types apply to a given type. This describes how the
/// type is encoded on the wire.
///
/// This enum contains all Serde Data Model types other than enum variants which exist in
/// [`DataModelVariant`], as well as a "Schema" Model Type, which maps to [`NamedType`].
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

    /// The `f64 Serde Data Model Type
    F64,

    /// The `char` Serde Data Model Type
    Char,

    /// The `String` Serde Data Model Type
    String,

    /// The `&[u8]` Serde Data Model Type
    ByteArray,

    /// The `Option<T>` Serde Data Model Type
    Option(&'static NamedType),

    /// The `()` Serde Data Model Type
    Unit,

    /// The "unit struct" Serde Data Model Type
    UnitStruct,

    /// The "newtype struct" Serde Data Model Type
    NewtypeStruct(&'static NamedType),

    /// The "Sequence" Serde Data Model Type
    Seq(&'static NamedType),

    /// The "Tuple" Serde Data Model Type
    Tuple(&'static [&'static NamedType]),

    /// The "Tuple Struct" Serde Data Model Type
    TupleStruct(&'static [&'static NamedType]),

    /// The "Map" Serde Data Model Type
    Map {
        /// The map "Key" type
        key: &'static NamedType,
        /// The map "Value" type
        val: &'static NamedType,
    },

    /// The "Struct" Serde Data Model Type
    Struct(&'static [&'static NamedValue]),

    /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
    Enum(&'static [&'static NamedVariant]),

    /// A NamedType/OwnedNamedType
    Schema,
}

/// This is similar to [`DataModelType`], however it only contains the potential Data Model Types
/// used as variants of an `enum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum DataModelVariant {
    /// The "unit variant" Serde Data Model Type
    UnitVariant,
    /// The "newtype variant" Serde Data Model Type
    NewtypeVariant(&'static NamedType),
    /// The "Tuple Variant" Serde Data Model Type
    TupleVariant(&'static [&'static NamedType]),
    /// The "Struct Variant" Serde Data Model Type
    StructVariant(&'static [&'static NamedValue]),
}

/// This represents a named struct field.
///
/// For example, in `struct Ex { a: u32 }` the field `a` would be reflected as
/// `NamedValue { name: "a", ty: DataModelType::U32 }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedValue {
    /// The name of this value
    pub name: &'static str,
    /// The type of this value
    pub ty: &'static NamedType,
}

/// An enum variant with a name, e.g. `T::Bar(...)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedVariant {
    /// The name of this variant
    pub name: &'static str,
    /// The type of this variant
    pub ty: &'static DataModelVariant,
}
