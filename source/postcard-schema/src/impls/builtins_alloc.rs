//! Implementations of the [`Schema`] trait for `alloc` types

use crate::{
    schema::{DataModelType, NamedType},
    Schema,
};

extern crate alloc;

impl<T: Schema> Schema for alloc::vec::Vec<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Vec<T>",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}

impl Schema for alloc::string::String {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "String",
        ty: &DataModelType::String,
    };
}

impl<K: Schema, V: Schema> Schema for alloc::collections::BTreeMap<K, V> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "BTreeMap<K, V>",
        ty: &DataModelType::Map {
            key: K::SCHEMA,
            val: V::SCHEMA,
        },
    };
}

impl<K: Schema> Schema for alloc::collections::BTreeSet<K> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "BTreeSet<K>",
        ty: &DataModelType::Seq(K::SCHEMA),
    };
}
