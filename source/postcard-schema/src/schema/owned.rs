//! Owned Schema version

use super::{DataModelType, DataModelVariant, NamedType, NamedValue, NamedVariant};
use serde::{Deserialize, Serialize};

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
extern crate alloc;

#[cfg(feature = "use-std")]
use std::{boxed::Box, collections::HashSet, string::String, vec::Vec};

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

// ---

/// The owned version of [`NamedType`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OwnedNamedType {
    /// The name of this type
    pub name: String,
    /// The type
    pub ty: OwnedDataModelType,
}

impl OwnedNamedType {
    /// Convert an [OwnedNamedType] to a pseudo-Rust type format
    pub fn to_pseudocode(&self) -> String {
        let mut buf = String::new();
        super::fmt::fmt_owned_nt_to_buf(self, &mut buf, true);
        buf
    }

    /// Collect all types used recursively by this type
    #[cfg(feature = "use-std")]
    pub fn all_used_types(&self) -> HashSet<OwnedNamedType> {
        let mut buf = HashSet::new();
        super::fmt::discover_tys(self, &mut buf);
        buf
    }
}

impl core::fmt::Display for OwnedNamedType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let pc = self.to_pseudocode();
        f.write_str(&pc)
    }
}

impl From<&NamedType> for OwnedNamedType {
    fn from(value: &NamedType) -> Self {
        Self {
            name: value.name.to_string(),
            ty: value.ty.into(),
        }
    }
}

impl crate::Schema for OwnedNamedType {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "OwnedNamedType",
        ty: &DataModelType::Schema,
    };
}

// ---

/// The owned version of [`DataModelType`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwnedDataModelType {
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
    Option(Box<OwnedNamedType>),

    /// The `()` Serde Data Model Type
    Unit,

    /// The "unit struct" Serde Data Model Type
    UnitStruct,

    /// The "newtype struct" Serde Data Model Type
    NewtypeStruct(Box<OwnedNamedType>),

    /// The "Sequence" Serde Data Model Type
    Seq(Box<OwnedNamedType>),

    /// The "Tuple" Serde Data Model Type
    Tuple(Vec<OwnedNamedType>),

    /// The "Tuple Struct" Serde Data Model Type
    TupleStruct(Vec<OwnedNamedType>),

    /// The "Map" Serde Data Model Type
    Map {
        /// The map "Key" type
        key: Box<OwnedNamedType>,
        /// The map "Value" type
        val: Box<OwnedNamedType>,
    },

    /// The "Struct" Serde Data Model Type
    Struct(Vec<OwnedNamedValue>),

    /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
    Enum(Vec<OwnedNamedVariant>),

    /// A NamedType/OwnedNamedType
    Schema,
}

impl From<&DataModelType> for OwnedDataModelType {
    fn from(other: &DataModelType) -> Self {
        match other {
            DataModelType::Bool => Self::Bool,
            DataModelType::I8 => Self::I8,
            DataModelType::U8 => Self::U8,
            DataModelType::I16 => Self::I16,
            DataModelType::I32 => Self::I32,
            DataModelType::I64 => Self::I64,
            DataModelType::I128 => Self::I128,
            DataModelType::U16 => Self::U16,
            DataModelType::U32 => Self::U32,
            DataModelType::U64 => Self::U64,
            DataModelType::U128 => Self::U128,
            DataModelType::Usize => Self::Usize,
            DataModelType::Isize => Self::Isize,
            DataModelType::F32 => Self::F32,
            DataModelType::F64 => Self::F64,
            DataModelType::Char => Self::Char,
            DataModelType::String => Self::String,
            DataModelType::ByteArray => Self::ByteArray,
            DataModelType::Option(o) => Self::Option(Box::new((*o).into())),
            DataModelType::Unit => Self::Unit,
            DataModelType::UnitStruct => Self::UnitStruct,
            DataModelType::NewtypeStruct(nts) => Self::NewtypeStruct(Box::new((*nts).into())),
            DataModelType::Seq(s) => Self::Seq(Box::new((*s).into())),
            DataModelType::Tuple(t) => Self::Tuple(t.iter().map(|i| (*i).into()).collect()),
            DataModelType::TupleStruct(ts) => {
                Self::TupleStruct(ts.iter().map(|i| (*i).into()).collect())
            }
            DataModelType::Map { key, val } => Self::Map {
                key: Box::new((*key).into()),
                val: Box::new((*val).into()),
            },
            DataModelType::Struct(s) => Self::Struct(s.iter().map(|i| (*i).into()).collect()),
            DataModelType::Enum(e) => Self::Enum(e.iter().map(|i| (*i).into()).collect()),
            DataModelType::Schema => Self::Schema,
        }
    }
}

// ---

/// The owned version of [`DataModelVariant`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwnedDataModelVariant {
    /// The "unit variant" Serde Data Model Type
    UnitVariant,
    /// The "newtype variant" Serde Data Model Type
    NewtypeVariant(Box<OwnedNamedType>),
    /// The "Tuple Variant" Serde Data Model Type
    TupleVariant(Vec<OwnedNamedType>),
    /// The "Struct Variant" Serde Data Model Type
    StructVariant(Vec<OwnedNamedValue>),
}

impl From<&DataModelVariant> for OwnedDataModelVariant {
    fn from(value: &DataModelVariant) -> Self {
        match value {
            DataModelVariant::UnitVariant => Self::UnitVariant,
            DataModelVariant::NewtypeVariant(d) => Self::NewtypeVariant(Box::new((*d).into())),
            DataModelVariant::TupleVariant(d) => {
                Self::TupleVariant(d.iter().map(|i| (*i).into()).collect())
            }
            DataModelVariant::StructVariant(d) => {
                Self::StructVariant(d.iter().map(|i| (*i).into()).collect())
            }
        }
    }
}

// ---

/// The owned version of [`NamedValue`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OwnedNamedValue {
    /// The name of this value
    pub name: String,
    /// The type of this value
    pub ty: OwnedNamedType,
}

impl From<&NamedValue> for OwnedNamedValue {
    fn from(value: &NamedValue) -> Self {
        Self {
            name: value.name.to_string(),
            ty: value.ty.into(),
        }
    }
}

// ---

/// The owned version of [`NamedVariant`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OwnedNamedVariant {
    /// The name of this variant
    pub name: String,
    /// The type of this variant
    pub ty: OwnedDataModelVariant,
}

impl From<&NamedVariant> for OwnedNamedVariant {
    fn from(value: &NamedVariant) -> Self {
        Self {
            name: value.name.to_string(),
            ty: value.ty.into(),
        }
    }
}
