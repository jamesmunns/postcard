//! Implementations of the [`Schema`] trait for the `indexmap` crate v2.0
use crate::{
    schema::{DataModelType, NamedType},
    Schema,
};

#[cfg_attr(docsrs, doc(cfg(feature = "indexmap-v2_0")))]
impl<K: Schema, V: Schema, S> Schema for indexmap_v2_0::IndexMap<K, V, S> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "IndexMap<K, V",
        ty: &DataModelType::Map {
            key: K::SCHEMA,
            val: V::SCHEMA,
        },
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "indexmap-v2_0")))]
impl<T: Schema, S> Schema for indexmap_v2_0::IndexSet<T, S> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "IndexSet<T>",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}
