//! Implementations of the [`Schema`] trait for `core` types
//!
//! These implementations are always available

use crate::{
    schema::{Data, DataModelType, NamedField, Variant},
    Schema,
};
use core::{
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    },
    ops::{Range, RangeFrom, RangeInclusive, RangeTo},
};

macro_rules! impl_schema {
    ($($t:ty: $sdm:expr),*) => {
        $(
            impl Schema for $t {
                const SCHEMA: &'static DataModelType = &$sdm;
            }
        )*
    };
    (tuple => [$(($($generic:ident),*)),*]) => {
        $(
            impl<$($generic: Schema),*> Schema for ($($generic,)*) {
                const SCHEMA: &'static DataModelType = &DataModelType::Tuple(&[$($generic::SCHEMA),*]);
            }
        )*
    };
}

impl_schema![
    u8: DataModelType::U8,
    NonZeroU8: DataModelType::U8,
    i8: DataModelType::I8,
    NonZeroI8: DataModelType::I8,
    bool: DataModelType::Bool,
    f32: DataModelType::F32,
    f64: DataModelType::F64,
    char: DataModelType::Char,
    str: DataModelType::String,
    (): DataModelType::Unit,
    i16: DataModelType::I16, i32: DataModelType::I32, i64: DataModelType::I64, i128: DataModelType::I128,
    u16: DataModelType::U16, u32: DataModelType::U32, u64: DataModelType::U64, u128: DataModelType::U128,
    NonZeroI16: DataModelType::I16, NonZeroI32: DataModelType::I32,
    NonZeroI64: DataModelType::I64, NonZeroI128: DataModelType::I128,
    NonZeroU16: DataModelType::U16, NonZeroU32: DataModelType::U32,
    NonZeroU64: DataModelType::U64, NonZeroU128: DataModelType::U128
];

impl_schema!(tuple => [
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F)
]);

impl<T: Schema> Schema for Option<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Option(T::SCHEMA);
}

impl<T: Schema, E: Schema> Schema for Result<T, E> {
    const SCHEMA: &'static DataModelType = &DataModelType::Enum {
        name: "Result<T, E>",
        variants: &[
            &Variant {
                name: "Ok",
                data: Data::Newtype(T::SCHEMA),
            },
            &Variant {
                name: "Err",
                data: Data::Newtype(E::SCHEMA),
            },
        ],
    };
}

impl<T: Schema + ?Sized> Schema for &'_ T {
    const SCHEMA: &'static DataModelType = T::SCHEMA;
}

impl<T: Schema> Schema for [T] {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq(T::SCHEMA);
}

impl<T: Schema, const N: usize> Schema for [T; N] {
    const SCHEMA: &'static DataModelType = &DataModelType::Tuple(&[T::SCHEMA; N]);
}

impl<T: Schema> Schema for Range<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "Range<T>",
        data: Data::Struct(&[
            &NamedField {
                name: "start",
                ty: T::SCHEMA,
            },
            &NamedField {
                name: "end",
                ty: T::SCHEMA,
            },
        ]),
    };
}

impl<T: Schema> Schema for RangeInclusive<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "RangeInclusive<T>",
        data: Data::Struct(&[
            &NamedField {
                name: "start",
                ty: T::SCHEMA,
            },
            &NamedField {
                name: "end",
                ty: T::SCHEMA,
            },
        ]),
    };
}

impl<T: Schema> Schema for RangeFrom<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "RangeFrom<T>",
        data: Data::Struct(&[&NamedField {
            name: "start",
            ty: T::SCHEMA,
        }]),
    };
}

impl<T: Schema> Schema for RangeTo<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "RangeTo<T>",
        data: Data::Struct(&[&NamedField {
            name: "end",
            ty: T::SCHEMA,
        }]),
    };
}
