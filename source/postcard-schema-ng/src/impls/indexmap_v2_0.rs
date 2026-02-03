//! Implementations of the [`Schema`] trait for the `indexmap` crate v2.0
use crate::{schema::DataModelType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "indexmap-v2_0")))]
impl<K: Schema, V: Schema, S> Schema for indexmap_v2_0::IndexMap<K, V, S> {
    const SCHEMA: &'static DataModelType = &DataModelType::Map {
        key: K::SCHEMA,
        val: V::SCHEMA,
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "indexmap-v2_0")))]
impl<T: Schema, S> Schema for indexmap_v2_0::IndexSet<T, S> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq(T::SCHEMA);
}
