//! Implementations of the [`Schema`] trait for the `heapless` crate v0.8

use crate::max_size::{bounded_seq_max, bounded_string_max};
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
    const MANUAL_MAX_SIZE: Option<usize> = bounded_seq_max::<Self, T, N>();
}
#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_8")))]
impl<const N: usize> Schema for heapless_v0_8::String<N> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "heapless::String<N>",
        ty: &DataModelType::String,
    };
    const MANUAL_MAX_SIZE: Option<usize> = bounded_string_max::<N>();
}

#[cfg(test)]
mod test {
    use crate::max_size::max_size;

    #[test]
    fn smoke() {
        assert_eq!(max_size::<heapless_v0_8::Vec<u8, 128>>(), Some(130));
        assert_eq!(max_size::<heapless_v0_8::String<128>>(), Some(130));
    }
}
