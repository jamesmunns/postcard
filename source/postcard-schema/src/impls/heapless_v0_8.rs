//! Implementations of the [`Schema`] trait for the `heapless` crate v0.8

use crate::{
    schema::{DataModelType, NamedType},
    Schema,
};

#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_8")))]
impl<T: Schema, const N: usize> Schema for heapless_v0_8::Vec<T, N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::Vec<T, N>",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}
#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_8")))]
impl<const N: usize> Schema for heapless_v0_8::String<N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::String<N>",
        ty: &DataModelType::String,
    };
}
