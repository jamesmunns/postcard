//! Implementations of the [`Schema`] trait for the `heapless` crate v0.7

use crate::{schema::DataModelType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_7")))]
impl<T: Schema, const N: usize> Schema for heapless_v0_7::Vec<T, N> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq(T::SCHEMA);
}

#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_7")))]
impl<const N: usize> Schema for heapless_v0_7::String<N> {
    const SCHEMA: &'static DataModelType = &DataModelType::String;
}
