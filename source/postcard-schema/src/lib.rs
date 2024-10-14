#![cfg_attr(not(any(test, feature = "use-std")), no_std)]
// #![warn(missing_docs)]

//! # Postcard Schema

pub mod schema_tys;

/// A trait that represents a compile time calculated schema
pub trait Schema {
    /// A recursive data structure that describes the schema of the given
    /// type.
    const SCHEMA: &'static schema_tys::NamedType;
}
