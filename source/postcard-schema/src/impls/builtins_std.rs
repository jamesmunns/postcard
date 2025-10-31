//! Implementations of the [`Schema`] trait for `std` types

use crate::{
    schema::{DataModelType, NamedType},
    Schema,
};

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for std::vec::Vec<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Vec<T>",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl Schema for std::string::String {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "String",
        ty: &DataModelType::String,
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "use-std")))]
impl Schema for std::path::PathBuf {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "PathBuf",
        ty: &DataModelType::String,
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema, V: Schema> Schema for std::collections::HashMap<K, V> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "HashMap<K, V>",
        ty: &DataModelType::Map {
            key: K::SCHEMA,
            val: V::SCHEMA,
        },
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema, V: Schema> Schema for std::collections::BTreeMap<K, V> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "BTreeMap<K, V>",
        ty: &DataModelType::Map {
            key: K::SCHEMA,
            val: V::SCHEMA,
        },
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema> Schema for std::collections::HashSet<K> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "HashSet<K>",
        ty: &DataModelType::Seq(K::SCHEMA),
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema> Schema for std::collections::BTreeSet<K> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "BTreeSet<K>",
        ty: &DataModelType::Seq(K::SCHEMA),
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for std::collections::VecDeque<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "VecDeque<T>",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for std::boxed::Box<T> {
    const SCHEMA: &'static NamedType = T::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: ?Sized + Schema + std::borrow::ToOwned> Schema for std::borrow::Cow<'_, T> {
    const SCHEMA: &'static NamedType = T::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for std::rc::Rc<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Rc<T>",
        ty: T::SCHEMA.ty,
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for std::sync::Arc<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Arc<T>",
        ty: T::SCHEMA.ty,
    };
}
