//! Implementations of the [`Schema`] trait for the `fixed` crate v1.0

use crate::{schema::NamedType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI8<Frac> {
    const SCHEMA: &'static NamedType = i8::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI16<Frac> {
    const SCHEMA: &'static NamedType = i16::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI32<Frac> {
    const SCHEMA: &'static NamedType = i32::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI64<Frac> {
    const SCHEMA: &'static NamedType = i64::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI128<Frac> {
    const SCHEMA: &'static NamedType = i128::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU8<Frac> {
    const SCHEMA: &'static NamedType = u8::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU16<Frac> {
    const SCHEMA: &'static NamedType = u16::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU32<Frac> {
    const SCHEMA: &'static NamedType = u32::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU64<Frac> {
    const SCHEMA: &'static NamedType = u64::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU128<Frac> {
    const SCHEMA: &'static NamedType = u128::SCHEMA;
}
