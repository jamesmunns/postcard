//! Implementations of the [`Schema`] trait for the `nalgebra` crate v0.33

use crate::{
    schema::{Data, DataModelType, NamedField},
    Schema,
};

#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v0_33")))]
impl<T, const R: usize, const C: usize, S> Schema
    for nalgebra_v0_33::Matrix<T, nalgebra_v0_33::Const<R>, nalgebra_v0_33::Const<C>, S>
where
    T: Schema + nalgebra_v0_33::Scalar,
    S: nalgebra_v0_33::base::storage::Storage<
        T,
        nalgebra_v0_33::Const<R>,
        nalgebra_v0_33::Const<C>,
    >,
{
    const SCHEMA: &'static DataModelType = &DataModelType::Tuple(flatten(&[[T::SCHEMA; R]; C]));
}

#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v0_33")))]
impl<T: Schema + nalgebra_v0_33::Scalar, const D: usize> Schema
    for nalgebra_v0_33::OPoint<T, nalgebra_v0_33::Const<D>>
where
    nalgebra_v0_33::base::default_allocator::DefaultAllocator:
        nalgebra_v0_33::base::allocator::Allocator<nalgebra_v0_33::Const<D>>,
{
    const SCHEMA: &'static DataModelType =
        nalgebra_v0_33::OVector::<T, nalgebra_v0_33::Const<D>>::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v0_33")))]
impl<T: Schema> Schema for nalgebra_v0_33::Unit<T> {
    const SCHEMA: &'static DataModelType = T::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v0_33")))]
impl<T: Schema + nalgebra_v0_33::Scalar> Schema for nalgebra_v0_33::Quaternion<T> {
    const SCHEMA: &'static DataModelType = nalgebra_v0_33::Vector4::<T>::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v0_33")))]
impl<T: Schema + nalgebra_v0_33::Scalar, const D: usize> Schema
    for nalgebra_v0_33::Translation<T, D>
{
    const SCHEMA: &'static DataModelType = nalgebra_v0_33::SVector::<T, D>::SCHEMA;
}

#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v0_33")))]
impl<T: Schema + nalgebra_v0_33::Scalar, R: Schema, const D: usize> Schema
    for nalgebra_v0_33::Isometry<T, R, D>
{
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "Isometry",
        data: Data::Struct(&[
            &NamedField {
                name: "rotation",
                ty: R::SCHEMA,
            },
            &NamedField {
                name: "translation",
                ty: nalgebra_v0_33::Translation::<T, D>::SCHEMA,
            },
        ]),
    };
}

/// Const version of the const-unstable [`<[[T; N]]>::as_flattened()`]
const fn flatten<T, const N: usize>(slice: &[[T; N]]) -> &[T] {
    const {
        assert!(size_of::<T>() != 0);
    }
    // SAFETY: `self.len() * N` cannot overflow because `self` is
    // already in the address space.
    let len = unsafe { slice.len().unchecked_mul(N) };
    // SAFETY: `[T]` is layout-identical to `[T; N]`
    unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), len) }
}

#[test]
fn flattened() {
    type T = nalgebra_v0_33::SMatrix<u8, 3, 3>;
    assert_eq!(T::SCHEMA, <[u8; 9]>::SCHEMA);
}

#[test]
fn smoke() {
    let x = nalgebra_v0_33::SMatrix::<u8, 3, 3>::new(1, 2, 3, 4, 5, 6, 7, 8, 9);
    let y = postcard::to_stdvec(&x).unwrap();
    assert_eq!(&[1, 4, 7, 2, 5, 8, 3, 6, 9], y.as_slice());
}
