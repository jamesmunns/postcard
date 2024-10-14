use crate::{
    schema::{NamedType, SdmTy},
    Schema,
};

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<T: Schema> Schema for std::vec::Vec<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Vec<T>",
        ty: &SdmTy::Seq(T::SCHEMA),
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl Schema for std::string::String {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "String",
        ty: &SdmTy::String,
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema, V: Schema> Schema for std::collections::HashMap<K, V> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "HashMap<K, V>",
        ty: &SdmTy::Map {
            key: K::SCHEMA,
            val: V::SCHEMA,
        },
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema, V: Schema> Schema for std::collections::BTreeMap<K, V> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "BTreeMap<K, V>",
        ty: &SdmTy::Map {
            key: K::SCHEMA,
            val: V::SCHEMA,
        },
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema> Schema for std::collections::HashSet<K> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "HashSet<K>",
        ty: &SdmTy::Seq(K::SCHEMA),
    };
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
impl<K: Schema> Schema for std::collections::BTreeSet<K> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "BTreeSet<K>",
        ty: &SdmTy::Seq(K::SCHEMA),
    };
}
