//! Implementations of the [`Schema`] trait for `core` types
//!
//! These implementations are always available

use crate::{
    schema::{DataModelType, DataModelVariant, NamedType, NamedValue, NamedVariant},
    Schema,
};
use core::{
    fmt::Arguments,
    marker::PhantomData,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    },
    ops::{Range, RangeFrom, RangeInclusive, RangeTo},
    time::Duration,
};

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
    (tuple => [$(($($generic:ident),*)),*]) => {
        $(
            impl<$($generic: Schema),*> Schema for ($($generic,)*) {
                const SCHEMA: &'static NamedType = &NamedType {
                    name: stringify!(($($generic,)*)),
                    ty: &DataModelType::Tuple(&[$($generic::SCHEMA),*]),
                };
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
    isize: DataModelType::Isize, usize: DataModelType::Usize,
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
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Option<T>",
        ty: &DataModelType::Option(T::SCHEMA),
    };
}
impl<T: Schema, E: Schema> Schema for Result<T, E> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Result<T, E>",
        ty: &DataModelType::Enum(&[
            &NamedVariant {
                name: "Ok",
                ty: &DataModelVariant::TupleVariant(&[T::SCHEMA]),
            },
            &NamedVariant {
                name: "Err",
                ty: &DataModelVariant::TupleVariant(&[E::SCHEMA]),
            },
        ]),
    };
}

impl<T: Schema + ?Sized> Schema for &'_ T {
    const SCHEMA: &'static NamedType = T::SCHEMA;
}

impl<T: Schema> Schema for [T] {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "[T]",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}
impl<T: Schema, const N: usize> Schema for [T; N] {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "[T; N]",
        ty: &DataModelType::Tuple(&[T::SCHEMA; N]),
    };
}

impl<T: Schema> Schema for Range<T> {
    const SCHEMA: &'static crate::schema::NamedType = &NamedType {
        name: "Range<T>",
        ty: &DataModelType::Struct(&[
            &NamedValue {
                name: "start",
                ty: T::SCHEMA,
            },
            &NamedValue {
                name: "end",
                ty: T::SCHEMA,
            },
        ]),
    };
}

impl<T: Schema> Schema for RangeInclusive<T> {
    const SCHEMA: &'static crate::schema::NamedType = &NamedType {
        name: "RangeInclusive<T>",
        ty: &DataModelType::Struct(&[
            &NamedValue {
                name: "start",
                ty: T::SCHEMA,
            },
            &NamedValue {
                name: "end",
                ty: T::SCHEMA,
            },
        ]),
    };
}

impl<T: Schema> Schema for RangeFrom<T> {
    const SCHEMA: &'static crate::schema::NamedType = &NamedType {
        name: "RangeFrom<T>",
        ty: &DataModelType::Struct(&[&NamedValue {
            name: "start",
            ty: T::SCHEMA,
        }]),
    };
}

impl<T: Schema> Schema for RangeTo<T> {
    const SCHEMA: &'static crate::schema::NamedType = &NamedType {
        name: "RangeTo<T>",
        ty: &DataModelType::Struct(&[&NamedValue {
            name: "end",
            ty: T::SCHEMA,
        }]),
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "core-net")))]
impl Schema for core::net::Ipv4Addr {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Ipv4Addr",
        ty: &DataModelType::Struct(&[&NamedValue {
            name: "octets",
            ty: <[u8; 4]>::SCHEMA,
        }]),
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "core-net")))]
impl Schema for core::net::Ipv6Addr {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Ipv6Addr",
        ty: &DataModelType::Struct(&[&NamedValue {
            name: "octets",
            ty: <[u8; 16]>::SCHEMA,
        }]),
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "core-net")))]
impl Schema for core::net::IpAddr {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "IpAddr",
        ty: &DataModelType::Enum(&[
            &NamedVariant {
                name: "V4",
                ty: &DataModelVariant::NewtypeVariant(core::net::Ipv4Addr::SCHEMA),
            },
            &NamedVariant {
                name: "V6",
                ty: &DataModelVariant::NewtypeVariant(core::net::Ipv6Addr::SCHEMA),
            },
        ]),
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "core-net")))]
impl Schema for core::net::SocketAddrV4 {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "SocketAddrV4",
        ty: &DataModelType::Struct(&[
            &NamedValue {
                name: "ip",
                ty: core::net::Ipv4Addr::SCHEMA,
            },
            &NamedValue {
                name: "port",
                ty: u16::SCHEMA,
            },
        ]),
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "core-net")))]
impl Schema for core::net::SocketAddrV6 {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "SocketAddrV6",
        ty: &DataModelType::Struct(&[
            &NamedValue {
                name: "ip",
                ty: core::net::Ipv6Addr::SCHEMA,
            },
            &NamedValue {
                name: "port",
                ty: u16::SCHEMA,
            },
            &NamedValue {
                name: "flowinfo",
                ty: u32::SCHEMA,
            },
            &NamedValue {
                name: "scope_id",
                ty: u32::SCHEMA,
            },
        ]),
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "core-net")))]
impl Schema for core::net::SocketAddr {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "SocketAddr",
        ty: &DataModelType::Enum(&[
            &NamedVariant {
                name: "V4",
                ty: &DataModelVariant::NewtypeVariant(core::net::SocketAddrV4::SCHEMA),
            },
            &NamedVariant {
                name: "V6",
                ty: &DataModelVariant::NewtypeVariant(core::net::SocketAddrV6::SCHEMA),
            },
        ]),
    };
}

impl<T: Schema> Schema for core::num::Wrapping<T> {
    const SCHEMA: &'static NamedType = T::SCHEMA;
}

#[cfg(feature = "core-num-saturating")]
#[cfg_attr(docsrs, doc(cfg(feature = "core-num-saturating")))]
impl<T: Schema> Schema for core::num::Saturating<T> {
    const SCHEMA: &'static NamedType = T::SCHEMA;
}

impl Schema for Duration {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Duration",
        ty: &DataModelType::Struct(&[
            &NamedValue {
                name: "secs",
                ty: u64::SCHEMA,
            },
            &NamedValue {
                name: "nanos",
                ty: u32::SCHEMA,
            },
        ]),
    };
}

impl<T: ?Sized> Schema for PhantomData<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "PhantomData",
        ty: &DataModelType::Unit,
    };
}

impl Schema for Arguments<'_> {
    const SCHEMA: &'static crate::schema::NamedType = &NamedType {
        name: "Arguments",
        ty: &DataModelType::String,
    };
}
