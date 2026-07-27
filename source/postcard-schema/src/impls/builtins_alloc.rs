//! Implementations of the [`Schema`] trait for `alloc` types

use crate::{
    schema::{DataModelType, NamedType},
    Schema,
};

extern crate alloc;

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for alloc::vec::Vec<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Vec<T>",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl Schema for alloc::string::String {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "String",
        ty: &DataModelType::String,
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema, V: Schema> Schema for alloc::collections::BTreeMap<K, V> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "BTreeMap<K, V>",
        ty: &DataModelType::Map {
            key: K::SCHEMA,
            val: V::SCHEMA,
        },
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema> Schema for alloc::collections::BTreeSet<K> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "BTreeSet<K>",
        ty: &DataModelType::Seq(K::SCHEMA),
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for alloc::collections::VecDeque<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "VecDeque<T>",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for alloc::boxed::Box<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Box<T>",
        ty: T::SCHEMA.ty,
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: ?Sized + Schema + alloc::borrow::ToOwned> Schema for alloc::borrow::Cow<'_, T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Cow<'_, T>",
        ty: T::SCHEMA.ty,
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for alloc::rc::Rc<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Rc<T>",
        ty: T::SCHEMA.ty,
    };
}
