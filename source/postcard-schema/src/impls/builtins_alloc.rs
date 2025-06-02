//! Implementations of the [`Schema`] trait for `alloc` types

use crate::{schema::DataModelType, Schema};

extern crate alloc;

impl<T: Schema> Schema for alloc::vec::Vec<T> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq(T::SCHEMA);
}

impl Schema for alloc::string::String {
    const SCHEMA: &'static DataModelType = &DataModelType::String;
}

impl<K: Schema, V: Schema> Schema for alloc::collections::BTreeMap<K, V> {
    const SCHEMA: &'static DataModelType = &DataModelType::Map {
        key: K::SCHEMA,
        val: V::SCHEMA,
    };
}

impl<K: Schema> Schema for alloc::collections::BTreeSet<K> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq(K::SCHEMA);
}

impl<T: ?Sized + Schema + alloc::borrow::ToOwned> Schema for alloc::borrow::Cow<'_, T> {
    const SCHEMA: &'static DataModelType = T::SCHEMA;
}
