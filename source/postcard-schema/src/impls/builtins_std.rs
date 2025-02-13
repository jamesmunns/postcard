//! Implementations of the [`Schema`] trait for `std` types

use crate::{schema::DataModelType, Schema};

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for std::vec::Vec<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq(T::SCHEMA);
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl Schema for std::string::String {
    const SCHEMA: &'static DataModelType = &DataModelType::String;
}

#[cfg_attr(docsrs, doc(cfg(feature = "use-std")))]
impl Schema for std::path::PathBuf {
    const SCHEMA: &'static DataModelType = &DataModelType::String;
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema, V: Schema> Schema for std::collections::HashMap<K, V> {
    const SCHEMA: &'static DataModelType = &DataModelType::Map {
        key: K::SCHEMA,
        val: V::SCHEMA,
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema, V: Schema> Schema for std::collections::BTreeMap<K, V> {
    const SCHEMA: &'static DataModelType = &DataModelType::Map {
        key: K::SCHEMA,
        val: V::SCHEMA,
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema> Schema for std::collections::HashSet<K> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq(K::SCHEMA);
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema> Schema for std::collections::BTreeSet<K> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq(K::SCHEMA);
}
