//! Owned Schema version

use crate::schema::size_as_varint_usize;

use super::{Data, DataModelType, NamedField, Variant};
use serde::{Deserialize, Serialize};

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
extern crate alloc;

#[cfg(feature = "use-std")]
use std::{boxed::Box, collections::HashSet, string::String};

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String};

// ---

impl OwnedDataModelType {
    /// Convert an `[OwnedDataModelType]` to a pseudo-Rust type format
    pub fn to_pseudocode(&self) -> String {
        let mut buf = String::new();
        super::fmt::fmt_owned_dmt_to_buf(self, &mut buf, true);
        buf
    }

    /// Collect all types used recursively by this type
    #[cfg(feature = "use-std")]
    pub fn all_used_types(&self) -> HashSet<Self> {
        let mut buf = HashSet::new();
        super::fmt::discover_tys(self, &mut buf);
        buf
    }
}

impl core::fmt::Display for OwnedDataModelType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let pc = self.to_pseudocode();
        f.write_str(&pc)
    }
}

impl crate::Schema for OwnedDataModelType {
    const SCHEMA: &'static DataModelType = &DataModelType::Schema;
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
    String {
        /// Maximum length (in bytes) of the string
        max_len: Option<usize>,
    },

    /// The `&[u8]` Serde Data Model Type
    ByteArray {
        /// Maximum length of byte array
        max_len: Option<usize>,
    },

    /// The `Option<T>` Serde Data Model Type
    Option(Box<Self>),

    /// The `()` Serde Data Model Type
    Unit,

    /// The "Sequence" Serde Data Model Type
    Seq {
        /// Items in the sequence
        element: Box<Self>,
        /// Maximum length of the sequence
        max_len: Option<usize>,
    },

    /// The "Tuple" Serde Data Model Type
    Tuple(Box<[Self]>),

    /// The "Map" Serde Data Model Type
    Map {
        /// The map "Key" type
        key: Box<Self>,
        /// The map "Value" type
        val: Box<Self>,
        /// Maximum length of K:V pairs
        max_len: Option<usize>,
    },

    /// One of the struct Serde Data Model types
    Struct {
        /// The name of this struct
        name: Box<str>,
        /// The data contained in this struct
        data: OwnedData,
    },

    /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
    Enum {
        /// The name of this struct
        name: Box<str>,
        /// The variants contained in this enum
        variants: Box<[OwnedVariant]>,
    },

    /// A [`DataModelType`]/[`OwnedDataModelType`]
    Schema,
}

const fn arr_max_size(data_model_types: &[OwnedDataModelType]) -> Option<usize> {
    let mut idx = 0;
    let mut sum = 0;
    while idx < data_model_types.len() {
        let Some(size) = data_model_types[idx].max_size() else {
            return None;
        };
        sum += size;
        idx += 1;
    }
    Some(sum)
}

max_size_dmt!(OwnedDataModelType);

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
            DataModelType::String { max_len: bounds } => Self::String { max_len: *bounds },
            DataModelType::ByteArray { max_len: bounds } => Self::ByteArray { max_len: *bounds },
            DataModelType::Option(o) => Self::Option(Box::new((*o).into())),
            DataModelType::Unit => Self::Unit,
            DataModelType::Seq {
                element: s,
                max_len: bounds,
            } => Self::Seq {
                element: Box::new((*s).into()),
                max_len: *bounds,
            },
            DataModelType::Tuple(t) => Self::Tuple(t.iter().map(|i| (*i).into()).collect()),
            DataModelType::Map {
                key,
                val,
                max_len: bounds,
            } => Self::Map {
                key: Box::new((*key).into()),
                val: Box::new((*val).into()),
                max_len: *bounds,
            },
            DataModelType::Struct { name, data } => Self::Struct {
                name: (*name).into(),
                data: data.into(),
            },
            DataModelType::Enum { name, variants } => Self::Enum {
                name: (*name).into(),
                variants: variants.iter().map(|i| (*i).into()).collect(),
            },
            DataModelType::Schema => Self::Schema,
        }
    }
}

// ---

/// The owned version of [`Data`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwnedData {
    /// The "Unit Struct" or "Unit Variant" Serde Data Model Type
    Unit,

    /// The "Newtype Struct" or "Newtype Variant" Serde Data Model Type
    Newtype(Box<OwnedDataModelType>),

    /// The "Tuple Struct" or "Tuple Variant" Serde Data Model Type
    Tuple(Box<[OwnedDataModelType]>),

    /// The "Struct" or "Struct Variant" Serde Data Model Type
    Struct(Box<[OwnedNamedField]>),
}

impl OwnedData {
    /// Max serialized size
    pub const fn max_size(&self) -> Option<usize> {
        match self {
            OwnedData::Unit => Some(0),
            OwnedData::Newtype(data_model_type) => data_model_type.max_size(),
            OwnedData::Tuple(data_model_types) => arr_max_size(data_model_types),
            OwnedData::Struct(named_fields) => {
                let mut idx = 0;
                let mut sum = 0;
                while idx < named_fields.len() {
                    let Some(size) = named_fields[idx].ty.max_size() else {
                        return None;
                    };
                    sum += size;
                    idx += 1;
                }
                Some(sum)
            }
        }
    }
}

impl From<&Data> for OwnedData {
    fn from(data: &Data) -> Self {
        match data {
            Data::Unit => Self::Unit,
            Data::Newtype(d) => Self::Newtype(Box::new((*d).into())),
            Data::Tuple(d) => Self::Tuple(d.iter().map(|i| (*i).into()).collect()),
            Data::Struct(d) => Self::Struct(d.iter().map(|i| (*i).into()).collect()),
        }
    }
}

// ---

/// The owned version of [`NamedField`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OwnedNamedField {
    /// The name of this value
    pub name: Box<str>,
    /// The type of this value
    pub ty: OwnedDataModelType,
}

impl From<&NamedField> for OwnedNamedField {
    fn from(value: &NamedField) -> Self {
        Self {
            name: value.name.into(),
            ty: value.ty.into(),
        }
    }
}

// ---

/// The owned version of [`Variant`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OwnedVariant {
    /// The name of this variant
    pub name: Box<str>,
    /// The data contained in this variant
    pub data: OwnedData,
}

impl From<&Variant> for OwnedVariant {
    fn from(value: &Variant) -> Self {
        Self {
            name: value.name.into(),
            data: (&value.data).into(),
        }
    }
}
