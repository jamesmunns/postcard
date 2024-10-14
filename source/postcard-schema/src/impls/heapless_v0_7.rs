use crate::{
    schema::{NamedType, DataModelType},
    Schema,
};

#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_7")))]
impl<T: Schema, const N: usize> Schema for heapless_v0_7::Vec<T, N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::Vec<T, N>",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}
#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_7")))]
impl<const N: usize> Schema for heapless_v0_7::String<N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::String<N>",
        ty: &DataModelType::String,
    };
}
