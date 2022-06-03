#![allow(missing_docs)]

#[cfg(feature = "derive")]
pub use postcard_derive::Schema;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum Varint {
    I16,
    I32,
    I64,
    I128,
    U16,
    U32,
    U64,
    U128,
    Usize,
    Isize,
}

/// Serde Data Model Types (and friends)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum SdmTy {
    Bool,
    I8,
    U8,
    Varint(Varint),
    F32,
    F64,
    Char,
    String,
    ByteArray,
    Option(&'static NamedType),
    Unit,
    UnitStruct,
    UnitVariant,
    NewtypeStruct(&'static NamedType),
    NewtypeVariant(&'static NamedType),
    Seq(&'static NamedType),
    Tuple(&'static [&'static NamedType]),
    TupleStruct(&'static [&'static NamedType]),
    TupleVariant(&'static [&'static NamedType]),
    Map {
        key: &'static NamedType,
        val: &'static NamedType,
    },
    Struct(&'static [&'static NamedValue]),
    StructVariant(&'static [&'static NamedValue]),

    // But also
    Enum(&'static [&'static NamedVariant]),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct NamedValue {
    pub name: &'static str,
    pub ty: &'static NamedType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct NamedType {
    pub name: &'static str,
    pub ty: &'static SdmTy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct NamedVariant {
    pub name: &'static str,
    pub ty: &'static SdmTy,
}

pub trait Schema {
    const SCHEMA: &'static NamedType;
}

macro_rules! impl_schema {
    ($($t:ty: $sdm:expr),*) => {
        $(
            impl Schema for $t {
                const SCHEMA: &'static NamedType = &NamedType {
                    name: stringify!($t),
                    ty: &$sdm,
                };
            }
        )*
    };
    (varint => [$($t:ty: $varint:expr),*]) => {
        impl_schema!($($t: SdmTy::Varint($varint)),*);
    };
    (tuple => [$(($($generic:ident),*)),*]) => {
        $(
            impl<$($generic: Schema),*> Schema for ($($generic,)*) {
                const SCHEMA: &'static NamedType = &NamedType {
                    name: stringify!(($($generic,)*)),
                    ty: &SdmTy::Tuple(&[$($generic::SCHEMA),*]),
                };
            }
        )*
    };
}

impl_schema![
    u8: SdmTy::U8,
    i8: SdmTy::I8,
    bool: SdmTy::Bool,
    f32: SdmTy::F32,
    f64: SdmTy::F64,
    char: SdmTy::Char,
    str: SdmTy::String,
    (): SdmTy::Unit
];
impl_schema!(varint => [
    i16: Varint::I16, i32: Varint::I32, i64: Varint::I64, i128: Varint::I128,
    u16: Varint::U16, u32: Varint::U32, u64: Varint::U64, u128: Varint::U128
]);
impl_schema!(tuple => [
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F)
]);

impl<T: Schema> Schema for Option<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Option<T>",
        ty: &SdmTy::Option(T::SCHEMA),
    };
}
impl<T: Schema, E: Schema> Schema for Result<T, E> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Result<T, E>",
        ty: &SdmTy::Enum(&[
            &NamedVariant {
                name: "Ok",
                ty: &SdmTy::TupleVariant(&[T::SCHEMA]),
            },
            &NamedVariant {
                name: "Err",
                ty: &SdmTy::TupleVariant(&[E::SCHEMA]),
            },
        ]),
    };
}

impl<T: Schema> Schema for &'_ T {
    const SCHEMA: &'static NamedType = T::SCHEMA;
}

impl<T: Schema> Schema for [T] {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "&[T]",
        ty: &SdmTy::Seq(T::SCHEMA),
    };
}
impl<T: Schema, const N: usize> Schema for [T; N] {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "[T; N]",
        ty: &SdmTy::Tuple(&[T::SCHEMA; N]),
    };
}

#[cfg(feature = "heapless")]
impl<T: Schema, const N: usize> Schema for heapless::Vec<T, N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::Vec<T, N>",
        ty: &SdmTy::Seq(T::SCHEMA),
    };
}
#[cfg(feature = "heapless")]
impl<const N: usize> Schema for heapless::String<N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::String<N>",
        ty: &SdmTy::String,
    };
}

#[cfg(feature = "use-std")]
impl<T: Schema> Schema for std::vec::Vec<T> {
    const TYPE: &'static NamedType = &NamedType {
        name: "Vec<T>",
        ty: &SdmTy::Seq(T::TYPE),
    };
}
#[cfg(feature = "alloc")]
impl<T: Schema> Schema for alloc::vec::Vec<T> {
    const TYPE: &'static NamedType = &NamedType {
        name: "Vec<T>",
        ty: &SdmTy::Seq(T::TYPE),
    };
}
