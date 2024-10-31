#![cfg_attr(not(any(test, feature = "use-std")), no_std)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! # Postcard Schema

pub mod impls;
pub mod max_size;
pub mod schema;

#[cfg(feature = "derive")]
pub use postcard_derive::Schema;

/// A trait that represents a compile time calculated schema
pub trait Schema {
    /// A recursive data structure that describes the schema of the given
    /// type.
    const SCHEMA: &'static schema::NamedType;
}
