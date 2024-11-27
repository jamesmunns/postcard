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
        ty: &DataModelType::Tuple(flatten(&[[T::SCHEMA; R]; C])),
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
    assert_eq!(T::SCHEMA.ty, <[u8; 9]>::SCHEMA.ty);
}

#[test]
fn smoke() {
    let x = nalgebra_v0_33::SMatrix::<u8, 3, 3>::new(1, 2, 3, 4, 5, 6, 7, 8, 9);
    let y = postcard::to_stdvec(&x).unwrap();
    assert_eq!(&[1, 4, 7, 2, 5, 8, 3, 6, 9], y.as_slice());
}
