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
    String {
        /// Upper bound of items in the sequence
        bounds: Option<usize>,
    },

    /// The `&[u8]` Serde Data Model Type
    ByteArray {
        /// Upper bound of items in the sequence
        bounds: Option<usize>,
    },

    /// The `Option<T>` Serde Data Model Type
    Option(&'static Self),

    /// The `()` Serde Data Model Type
    Unit,

    /// The "Sequence" Serde Data Model Type
    Seq {
        /// Items in the sequence
        item: &'static Self,
        /// Upper bound of items in the sequence
        bounds: Option<usize>,
    },

    /// The "Tuple" Serde Data Model Type
    Tuple(&'static [&'static Self]),

    /// The "Map" Serde Data Model Type
    Map {
        /// The map "Key" type
        key: &'static Self,
        /// The map "Value" type
        val: &'static Self,
        /// Bounds
        bounds: Option<usize>,
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

const fn arr_max_size(data_model_types: &[&DataModelType]) -> Option<usize> {
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

pub(crate) const fn size_as_varint_usize(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let ttl_bits = usize::BITS as usize;
    let ldg_bits = n.leading_zeros() as usize;
    let used_bits = ttl_bits - ldg_bits;
    used_bits.div_ceil(7)
}

macro_rules! max_size_dmt {
    ($kind:ty) => {
        impl $kind {
            /// Max serialized size
            pub const fn max_size(&self) -> Option<usize> {
                use $kind::*;
                match self {
                    Bool => Some(1),
                    I8 => Some(1),
                    U8 => Some(1),
                    I16 => Some(postcard_core::varint::varint_max::<i16>()),
                    I32 => Some(postcard_core::varint::varint_max::<i32>()),
                    I64 => Some(postcard_core::varint::varint_max::<i64>()),
                    I128 => Some(postcard_core::varint::varint_max::<i128>()),
                    U16 => Some(postcard_core::varint::varint_max::<u16>()),
                    U32 => Some(postcard_core::varint::varint_max::<i32>()),
                    U64 => Some(postcard_core::varint::varint_max::<u64>()),
                    U128 => Some(postcard_core::varint::varint_max::<u128>()),
                    Usize => Some(postcard_core::varint::varint_max::<usize>()),
                    Isize => Some(postcard_core::varint::varint_max::<isize>()),
                    F32 => Some(4),
                    F64 => Some(8),
                    Char => Some(5),
                    String { bounds } | ByteArray { bounds } => {
                        if let Some(bound) = bounds {
                            Some(size_as_varint_usize(*bound) + *bound)
                        } else {
                            None
                        }
                    }
                    Option(data_model_type) => {
                        if let Some(bound) = data_model_type.max_size() {
                            Some(1 + bound)
                        } else {
                            None
                        }
                    }
                    Unit => Some(0),
                    Seq { item, bounds } => {
                        let Some(bound) = bounds else {
                            return None;
                        };
                        let Some(size) = item.max_size() else {
                            return None;
                        };
                        Some((*bound) * size)
                    }
                    Tuple(data_model_types) => arr_max_size(data_model_types),
                    Map { key, val, bounds } => {
                        let Some(bound) = bounds else {
                            return None;
                        };
                        let Some(ksize) = key.max_size() else {
                            return None;
                        };
                        let Some(vsize) = val.max_size() else {
                            return None;
                        };
                        let pair = ksize + vsize;
                        let pairs = (*bound) * pair;
                        Some(size_as_varint_usize(*bound) + pairs)
                    }
                    Struct { name: _, data } => data.max_size(),
                    Enum { name: _, variants } => {
                        let mut idx = 0;
                        let disc = if variants.is_empty() {
                            1
                        } else {
                            size_as_varint_usize(variants.len() - 1)
                        };
                        let mut max = 0;
                        while idx < variants.len() {
                            let Some(size) = variants[idx].data.max_size() else {
                                return None;
                            };
                            if size > max {
                                max = size;
                            }
                            idx += 1;
                        }
                        Some(disc + max)
                    }
                    Schema => todo!(),
                }
            }
        }
    };
}
max_size_dmt!(DataModelType);

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

impl Data {
    /// Max serialized size
    pub const fn max_size(&self) -> Option<usize> {
        match self {
            Data::Unit => Some(0),
            Data::Newtype(data_model_type) => data_model_type.max_size(),
            Data::Tuple(data_model_types) => arr_max_size(data_model_types),
            Data::Struct(named_fields) => {
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

#[cfg(any(feature = "use-std", feature = "alloc"))]
pub mod owned;
