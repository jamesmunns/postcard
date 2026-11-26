//! Implementations of the [`Schema`] trait for the `uom` crate v0.38

use crate::{schema::DataModelType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "uom-v0_38")))]
impl<D, U, V> Schema for uom_v0_38::si::Quantity<D, U, V>
where
    D: uom_v0_38::si::Dimension + ?Sized,
    U: uom_v0_38::si::Units<V> + ?Sized,
    V: uom_v0_38::num::Num + uom_v0_38::Conversion<V> + Schema,
{
    const SCHEMA: &'static DataModelType = V::SCHEMA;
}
