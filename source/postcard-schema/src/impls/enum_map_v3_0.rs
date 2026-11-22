//! Implementations of the [`Schema`] trait for the `enum-map` crate v3

use crate::{schema::DataModelType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "enum-map-v3_0")))]
impl<K: enum_map_v3_0::Enum<Array<V>: Schema>, V: Schema> Schema for enum_map_v3_0::EnumMap<K, V> {
    // K::Array is guaranteed to be [T; N]
    const SCHEMA: &'static DataModelType = K::Array::SCHEMA;
}
