use serde::{Deserialize, Serialize};
use postcard_derive::Schema;

/// A schema type representing a variably encoded integer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Varint {
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
}

impl Schema for Varint {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Varint",
        ty: &SdmTy::Enum(&[
            &NamedVariant { name: "I16", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "I32", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "I64", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "I128", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "U16", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "U32", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "U64", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "U128", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "Usize", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "Isize", ty: &SdmTy::UnitVariant },
        ]),
    };
}

/// Serde Data Model Types (and friends)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum SdmTy {
    /// The `bool` Serde Data Model Type
    Bool,

    /// The `i8` Serde Data Model Type
    I8,

    /// The `u8` Serde Data Model Type
    U8,

    /// The Serde Data Model Type for variably length encoded integers
    Varint(Varint),

    /// The `f32` Serde Data Model Type
    F32,

    /// The `f64 Serde Data Model Type
    F64,

    /// The `char` Serde Data Model Type
    Char,

    /// The `String` Serde Data Model Type
    String,

    /// The `[u8; N]` Serde Data Model Type
    ByteArray,

    /// The `Option<T>` Serde Data Model Type
    Option(&'static NamedType),

    /// The `()` Serde Data Model Type
    Unit,

    /// The "unit struct" Serde Data Model Type
    UnitStruct,

    /// The "unit variant" Serde Data Model Type
    UnitVariant,

    /// The "newtype struct" Serde Data Model Type
    NewtypeStruct(&'static NamedType),

    /// The "newtype variant" Serde Data Model Type
    NewtypeVariant(&'static NamedType),

    /// The "Sequence" Serde Data Model Type
    Seq(&'static NamedType),

    /// The "Tuple" Serde Data Model Type
    Tuple(&'static [&'static NamedType]),

    /// The "Tuple Struct" Serde Data Model Type
    TupleStruct(&'static [&'static NamedType]),

    /// The "Tuple Variant" Serde Data Model Type
    TupleVariant(&'static [&'static NamedType]),

    /// The "Map" Serde Data Model Type
    Map {
        /// The map "Key" type
        key: &'static NamedType,
        /// The map "Value" type
        val: &'static NamedType,
    },

    /// The "Struct" Serde Data Model Type
    Struct(&'static [&'static NamedValue]),

    /// The "Struct Variant" Serde Data Model Type
    StructVariant(&'static [&'static NamedValue]),

    /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
    Enum(&'static [&'static NamedVariant]),
}

impl Schema for SdmTy {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "SdmTy",
        ty: &SdmTy::Enum(&[
            &NamedVariant { name: "Bool", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "I8", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "U8", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "Varint", ty: &SdmTy::NewtypeVariant(
                NamedType::SCHEMA
            )},
            &NamedVariant { name: "F32", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "F64", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "Char", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "String", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "ByteArray", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "Option", ty: &SdmTy::NewtypeVariant(
                NamedType::SCHEMA
            )},
            &NamedVariant { name: "Unit", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "UnitStruct", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "UnitVariant", ty: &SdmTy::UnitVariant },
            &NamedVariant { name: "NewtypeStruct", ty: &SdmTy::NewtypeVariant(
                NamedType::SCHEMA
            )},
            &NamedVariant { name: "NewtypeVariant", ty: &SdmTy::NewtypeVariant(
                NamedType::SCHEMA
            )},
            &NamedVariant { name: "Seq", ty: &SdmTy::NewtypeVariant(
                NamedType::SCHEMA
            )},
            &NamedVariant { name: "Tuple", ty: &SdmTy::NewtypeVariant(
                <[NamedType]>::SCHEMA,
            )},
            &NamedVariant { name: "TupleStruct", ty: &SdmTy::NewtypeVariant(
                <[NamedType]>::SCHEMA,
            )},
            &NamedVariant { name: "TupleVariant", ty: &SdmTy::NewtypeVariant(
                <[NamedType]>::SCHEMA,
            )},
            &NamedVariant { name: "Map", ty: &SdmTy::StructVariant(&[
                &NamedValue { name: "key", ty: NamedType::SCHEMA },
                &NamedValue { name: "val", ty: NamedType::SCHEMA },
            ])},
            &NamedVariant { name: "Struct", ty: &SdmTy::NewtypeVariant(
                <[NamedValue]>::SCHEMA,
            )},
            &NamedVariant { name: "StructVariant", ty: &SdmTy::NewtypeVariant(
                <[NamedValue]>::SCHEMA,
            )},
            &NamedVariant { name: "Enum", ty: &SdmTy::NewtypeVariant(
                <[NamedVariant]>::SCHEMA,
            )},
        ]),
    };
}

/// A data type with a name - e.g. a field of a Struct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedValue {
    /// The name of this value
    pub name: &'static str,
    /// The type of this value
    pub ty: &'static NamedType,
}

impl Schema for NamedValue {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "NamedValue",
        ty: &SdmTy::Struct(&[
            &NamedValue { name: "name", ty: str::SCHEMA },
            &NamedValue { name: "ty", ty: NamedType::SCHEMA },
        ]),
    };
}

/// A data type - e.g. a custom `struct Foo{ ... }` type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedType {
    /// The name of this type
    pub name: &'static str,
    /// The type
    pub ty: &'static SdmTy,
}

impl Schema for NamedType {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "NamedType",
        ty: &SdmTy::Struct(&[
            &NamedValue { name: "name", ty: str::SCHEMA },
            &NamedValue { name: "ty", ty: SdmTy::SCHEMA },
        ]),
    };
}

/// An enum variant with a name, e.g. `T::Bar(...)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedVariant {
    /// The name of this variant
    pub name: &'static str,
    /// The type of this variant
    pub ty: &'static SdmTy,
}

impl Schema for NamedVariant {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "SdmTy",
        ty: &SdmTy::Struct(&[
            // TODO stuff should be here
        ]),
    };
}

/// A trait that represents a compile time calculated schema
pub trait Schema {
    /// A recursive data structure that describes the schema of the given
    /// type.
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

impl<T: Schema + ?Sized> Schema for &'_ T {
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
#[cfg_attr(docsrs, doc(cfg(feature = "heapless")))]
impl<T: Schema, const N: usize> Schema for heapless::Vec<T, N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::Vec<T, N>",
        ty: &SdmTy::Seq(T::SCHEMA),
    };
}
#[cfg(feature = "heapless")]
#[cfg_attr(docsrs, doc(cfg(feature = "heapless")))]
impl<const N: usize> Schema for heapless::String<N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::String<N>",
        ty: &SdmTy::String,
    };
}

#[cfg(feature = "use-std")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for std::vec::Vec<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Vec<T>",
        ty: &SdmTy::Seq(T::SCHEMA),
    };
}

#[cfg(feature = "use-std")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl Schema for std::string::String {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "String",
        ty: &SdmTy::String,
    };
}

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
impl<T: Schema> Schema for alloc::vec::Vec<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Vec<T>",
        ty: &SdmTy::Seq(T::SCHEMA),
    };
}

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
impl Schema for alloc::string::String {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "String",
        ty: &SdmTy::String,
    };
}

#[cfg(any(feature = "use-std", feature = "alloc"))]
pub(crate) mod owned {
    use super::*;

    #[cfg(feature = "use-std")]
    use std::{boxed::Box, string::String, vec::Vec};

    #[cfg(all(not(feature = "use-std"), feature = "alloc"))]
    use alloc::{
        boxed::Box,
        string::{String, ToString},
        vec::Vec,
    };

    /// Serde Data Model Types (and friends)
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum OwnedSdmTy {
        /// The `bool` Serde Data Model Type
        Bool,

        /// The `i8` Serde Data Model Type
        I8,

        /// The `u8` Serde Data Model Type
        U8,

        /// The Serde Data Model Type for variably length encoded integers
        Varint(Varint),

        /// The `f32` Serde Data Model Type
        F32,

        /// The `f64 Serde Data Model Type
        F64,

        /// The `char` Serde Data Model Type
        Char,

        /// The `String` Serde Data Model Type
        String,

        /// The `[u8; N]` Serde Data Model Type
        ByteArray,

        /// The `Option<T>` Serde Data Model Type
        Option(Box<OwnedNamedType>),

        /// The `()` Serde Data Model Type
        Unit,

        /// The "unit struct" Serde Data Model Type
        UnitStruct,

        /// The "unit variant" Serde Data Model Type
        UnitVariant,

        /// The "newtype struct" Serde Data Model Type
        NewtypeStruct(Box<OwnedNamedType>),

        /// The "newtype variant" Serde Data Model Type
        NewtypeVariant(Box<OwnedNamedType>),

        /// The "Sequence" Serde Data Model Type
        Seq(Box<OwnedNamedType>),

        /// The "Tuple" Serde Data Model Type
        Tuple(Vec<OwnedNamedType>),

        /// The "Tuple Struct" Serde Data Model Type
        TupleStruct(Vec<OwnedNamedType>),

        /// The "Tuple Variant" Serde Data Model Type
        TupleVariant(Vec<OwnedNamedType>),

        /// The "Map" Serde Data Model Type
        Map {
            /// The map "Key" type
            key: Box<OwnedNamedType>,
            /// The map "Value" type
            val: Box<OwnedNamedType>,
        },

        /// The "Struct" Serde Data Model Type
        Struct(Vec<OwnedNamedValue>),

        /// The "Struct Variant" Serde Data Model Type
        StructVariant(Vec<OwnedNamedValue>),

        /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
        Enum(Vec<OwnedNamedVariant>),
    }

    impl From<&SdmTy> for OwnedSdmTy {
        fn from(other: &SdmTy) -> Self {
            match other {
                SdmTy::Bool => Self::Bool,
                SdmTy::I8 => Self::I8,
                SdmTy::U8 => Self::U8,
                SdmTy::Varint(v) => Self::Varint(*v),
                SdmTy::F32 => Self::F32,
                SdmTy::F64 => Self::F64,
                SdmTy::Char => Self::Char,
                SdmTy::String => Self::String,
                SdmTy::ByteArray => Self::ByteArray,
                SdmTy::Option(o) => Self::Option(Box::new((*o).into())),
                SdmTy::Unit => Self::Unit,
                SdmTy::UnitStruct => Self::UnitStruct,
                SdmTy::UnitVariant => Self::UnitVariant,
                SdmTy::NewtypeStruct(nts) => Self::NewtypeStruct(Box::new((*nts).into())),
                SdmTy::NewtypeVariant(ntv) => Self::NewtypeVariant(Box::new((*ntv).into())),
                SdmTy::Seq(s) => Self::Seq(Box::new((*s).into())),
                SdmTy::Tuple(t) => Self::Tuple(t.iter().map(|i| (*i).into()).collect()),
                SdmTy::TupleStruct(ts) => {
                    Self::TupleStruct(ts.iter().map(|i| (*i).into()).collect())
                }
                SdmTy::TupleVariant(tv) => {
                    Self::TupleVariant(tv.iter().map(|i| (*i).into()).collect())
                }
                SdmTy::Map { key, val } => Self::Map {
                    key: Box::new((*key).into()),
                    val: Box::new((*val).into()),
                },
                SdmTy::Struct(s) => Self::Struct(s.iter().map(|i| (*i).into()).collect()),
                SdmTy::StructVariant(sv) => {
                    Self::StructVariant(sv.iter().map(|i| (*i).into()).collect())
                }
                SdmTy::Enum(e) => Self::Enum(e.iter().map(|i| (*i).into()).collect()),
            }
        }
    }

    /// A data type with a name - e.g. a field of a Struct
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

    /// A data type - e.g. a custom `struct Foo{ ... }` type
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct OwnedNamedType {
        /// The name of this type
        pub name: String,
        /// The type
        pub ty: OwnedSdmTy,
    }

    impl From<&NamedType> for OwnedNamedType {
        fn from(value: &NamedType) -> Self {
            Self {
                name: value.name.to_string(),
                ty: value.ty.into(),
            }
        }
    }

    /// An enum variant with a name, e.g. `T::Bar(...)`
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct OwnedNamedVariant {
        /// The name of this variant
        pub name: String,
        /// The type of this variant
        pub ty: OwnedSdmTy,
    }

    impl From<&NamedVariant> for OwnedNamedVariant {
        fn from(value: &NamedVariant) -> Self {
            Self {
                name: value.name.to_string(),
                ty: value.ty.into(),
            }
        }
    }
}
