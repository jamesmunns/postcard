//! Implementations of the [`Schema`] trait for the `fixed` crate v1.0

use crate::{schema::DataModelType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI8<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::I8;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI16<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::I16;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI32<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::I32;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI64<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::I64;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedI128<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::I128;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU8<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::U8;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU16<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::U16;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU32<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::U32;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU64<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::U64;
}

#[cfg_attr(docsrs, doc(cfg(feature = "fixed-v1_0")))]
impl<Frac> Schema for fixed_v1_0::FixedU128<Frac> {
    const SCHEMA: &'static DataModelType = &DataModelType::U128;
}
