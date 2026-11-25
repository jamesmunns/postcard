//! Implementations of the [`Schema`] trait for `core` types
//!
//! These implementations are always available

use crate::{
    schema::{Data, DataModelType, NamedField, Variant},
    Schema,
};
use core::{
    fmt::Arguments,
    marker::PhantomData,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
    ops::{Range, RangeFrom, RangeInclusive, RangeTo},
    time::Duration,
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
    str: DataModelType::String { max_len: None },
    (): DataModelType::Unit,
    i16: DataModelType::I16, i32: DataModelType::I32, i64: DataModelType::I64, i128: DataModelType::I128,
    u16: DataModelType::U16, u32: DataModelType::U32, u64: DataModelType::U64, u128: DataModelType::U128,
    NonZeroI16: DataModelType::I16, NonZeroI32: DataModelType::I32,
    NonZeroI64: DataModelType::I64, NonZeroI128: DataModelType::I128,
    NonZeroU16: DataModelType::U16, NonZeroU32: DataModelType::U32,
    NonZeroU64: DataModelType::U64, NonZeroU128: DataModelType::U128
];

// serde has no `usize` or `isize`: their `Serialize` impls call `serialize_u64`
// and `serialize_i64`, so we use that for their schema impls
impl_schema![
    usize: DataModelType::U64, NonZeroUsize: DataModelType::U64,
    isize: DataModelType::I64, NonZeroIsize: DataModelType::I64
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
        name: "Result",
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
    const SCHEMA: &'static DataModelType = &DataModelType::Seq {
        element: T::SCHEMA,
        max_len: None,
    };
}

impl<T: Schema, const N: usize> Schema for [T; N] {
    const SCHEMA: &'static DataModelType = &DataModelType::Tuple(&[T::SCHEMA; N]);
}

impl<T: Schema> Schema for Range<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "Range",
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
        name: "RangeInclusive",
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
        name: "RangeFrom",
        data: Data::Struct(&[&NamedField {
            name: "start",
            ty: T::SCHEMA,
        }]),
    };
}

impl<T: Schema> Schema for RangeTo<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "RangeTo",
        data: Data::Struct(&[&NamedField {
            name: "end",
            ty: T::SCHEMA,
        }]),
    };
}

// serde writes the octets as a `[u8; 4]`
impl Schema for core::net::Ipv4Addr {
    const SCHEMA: &'static DataModelType = <[u8; 4]>::SCHEMA;
}

// serde writes the octets as a `[u8; 16]`
impl Schema for core::net::Ipv6Addr {
    const SCHEMA: &'static DataModelType = <[u8; 16]>::SCHEMA;
}

impl Schema for core::net::IpAddr {
    const SCHEMA: &'static DataModelType = &DataModelType::Enum {
        name: "IpAddr",
        variants: &[
            &Variant {
                name: "V4",
                data: Data::Newtype(core::net::Ipv4Addr::SCHEMA),
            },
            &Variant {
                name: "V6",
                data: Data::Newtype(core::net::Ipv6Addr::SCHEMA),
            },
        ],
    };
}

// serde writes an `(ip, port)` tuple
impl Schema for core::net::SocketAddrV4 {
    const SCHEMA: &'static DataModelType = <(core::net::Ipv4Addr, u16)>::SCHEMA;
}

// serde writes an `(ip, port)` tuple, without the flow info and scope ID
impl Schema for core::net::SocketAddrV6 {
    const SCHEMA: &'static DataModelType = <(core::net::Ipv6Addr, u16)>::SCHEMA;
}

impl Schema for core::net::SocketAddr {
    const SCHEMA: &'static DataModelType = &DataModelType::Enum {
        name: "SocketAddr",
        variants: &[
            &Variant {
                name: "V4",
                data: Data::Newtype(core::net::SocketAddrV4::SCHEMA),
            },
            &Variant {
                name: "V6",
                data: Data::Newtype(core::net::SocketAddrV6::SCHEMA),
            },
        ],
    };
}

impl<T: Schema> Schema for core::num::Wrapping<T> {
    const SCHEMA: &'static DataModelType = T::SCHEMA;
}

impl<T: Schema> Schema for core::num::Saturating<T> {
    const SCHEMA: &'static DataModelType = T::SCHEMA;
}

impl Schema for Duration {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "Duration",
        data: Data::Struct(&[
            &NamedField {
                name: "secs",
                ty: u64::SCHEMA,
            },
            &NamedField {
                name: "nanos",
                ty: u32::SCHEMA,
            },
        ]),
    };
}

impl<T: ?Sized> Schema for PhantomData<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "PhantomData",
        data: Data::Unit,
    };
}

impl Schema for Arguments<'_> {
    const SCHEMA: &'static crate::schema::DataModelType = &DataModelType::String { max_len: None };
}
