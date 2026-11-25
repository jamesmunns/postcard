//! Implementations of the [`Schema`] trait for the `fixed` crate v1.0
//!
//! Without its `serde-str` feature, `fixed` serializes each number as a struct
//! with one field, `bits`, and that is the schema here. With `serde-str`, it
//! serializes just the bits instead. Both have the same encoding, but the
//! schema here only matches the first.

use crate::{
    schema::{Data, DataModelType, NamedField},
    Schema,
};

macro_rules! fixed_schema {
    ($($ty:ident => $bits:ident),* $(,)?) => {
        $(
            #[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
            impl<Frac> Schema for fixed_v1_0::$ty<Frac> {
                const SCHEMA: &'static DataModelType = &DataModelType::Struct {
                    name: stringify!($ty),
                    data: Data::Struct(&[&NamedField {
                        name: "bits",
                        ty: &DataModelType::$bits,
                    }]),
                };
            }
        )*
    };
}

fixed_schema![
    FixedI8 => I8,
    FixedI16 => I16,
    FixedI32 => I32,
    FixedI64 => I64,
    FixedI128 => I128,
    FixedU8 => U8,
    FixedU16 => U16,
    FixedU32 => U32,
    FixedU64 => U64,
    FixedU128 => U128,
];
