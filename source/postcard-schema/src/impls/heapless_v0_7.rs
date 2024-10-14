use crate::{
    schema::{NamedType, SdmTy},
    Schema,
};

#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_7")))]
impl<T: Schema, const N: usize> Schema for heapless_v0_7::Vec<T, N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::Vec<T, N>",
        ty: &SdmTy::Seq(T::SCHEMA),
    };
}
#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_7")))]
impl<const N: usize> Schema for heapless_v0_7::String<N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::String<N>",
        ty: &SdmTy::String,
    };
}
