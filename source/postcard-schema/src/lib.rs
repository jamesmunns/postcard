#![cfg_attr(not(any(test, feature = "use-std")), no_std)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! # Postcard Schema

pub mod impls;
pub mod max_size;
pub mod schema;
pub mod const_helpers;

#[cfg(feature = "derive")]
pub use postcard_derive::Schema;

/// A trait that represents a compile time calculated schema
pub trait Schema {
    /// A recursive data structure that describes the schema of the given
    /// type.
    const SCHEMA: &'static schema::NamedType;

    /// A manually calculated maximum size hint.
    ///
    /// This is useful in the case that a foreign type is expressed
    /// in some form that has a technically unbounded max size, for
    /// example a byte slice, but in reality it never sends more than
    /// some calcuable size.
    ///
    /// A good example is `heapless::Vec<T, N>`, which is expressed
    /// on the wire as a `Seq(N)`, which is unbounded, however the
    /// true max size is the length to store N as a varint plus
    /// `N * max_size::<T>`, which is calcuable.
    ///
    /// This value should not typically be used directly, instead use
    /// [`max_size()`][crate::max_size::max_size()], which will EITHER
    /// use the manual max size, or calculate the max size from the
    /// schema.
    ///
    /// You must not rely on this value for safety reasons, as implementations
    /// could be wrong.
    const MANUAL_MAX_SIZE: Option<usize> = None;
}
