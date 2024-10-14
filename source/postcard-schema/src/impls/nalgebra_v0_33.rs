//! Implementations of the [`Schema`] trait for the `nalgebra` crate v0.33

use crate::{
    schema::{DataModelType, NamedType},
    Schema,
};

#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v0_33")))]
impl<T, const R: usize, const C: usize> Schema
    for nalgebra_v0_33::Matrix<
        T,
        nalgebra_v0_33::Const<R>,
        nalgebra_v0_33::Const<C>,
        nalgebra_v0_33::ArrayStorage<T, R, C>,
    >
where
    T: Schema + nalgebra_v0_33::Scalar,
{
    const SCHEMA: &'static NamedType = &NamedType {
        name: "nalgebra::Matrix<T, R, C, ArrayStorage<T, R, C>>",
        ty: &DataModelType::Seq(T::SCHEMA),
    };
}
