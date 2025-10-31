//! Implementations of the [`Schema`] trait for the `num-complex` crate v0.4

use crate::{schema::NamedType, Schema};

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "num-complex-v0_4",
        feature = "nalgebra-v0_33",
        feature = "nalgebra-v0_34"
    )))
)]
impl<T: Schema> Schema for num_complex_v0_4::Complex<T> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Complex<T>",
        ty: <[T; 2]>::SCHEMA.ty,
    };
}
