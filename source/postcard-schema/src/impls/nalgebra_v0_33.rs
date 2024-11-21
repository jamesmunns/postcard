//! Implementations of the [`Schema`] trait for the `nalgebra` crate v0.33

use crate::{
    schema::NamedType,
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
    /// Warning!
    const SCHEMA: &'static NamedType = &NamedType {
        name: "nalgebra::Matrix<T, R, C, ArrayStorage<T, R, C>>",
        // NOTE: This is not TECHNICALLY correct.
        ty: <[[T; R]; C]>::SCHEMA.ty,
    };
}

#[cfg(test)]
mod test {
    use nalgebra_v0_33::Const;
    #[test]
    fn smoke() {
        let x = nalgebra_v0_33::Matrix::<u8, Const<3>, Const<3>, _>::new(1, 2, 3, 4, 5, 6, 7, 8, 9);
        let y = postcard::to_stdvec(&x).unwrap();
        assert_eq!(&[1, 4, 7, 2, 5, 8, 3, 6, 9], y.as_slice());
    }
}
