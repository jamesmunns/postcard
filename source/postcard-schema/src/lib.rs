#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! # Postcard Schema
//!
//! Schemas and keys for types sent with [postcard].
//!
//! A schema ([`DataModelType`](schema::DataModelType)) describes a type the way
//! serde sees it: the calls its `Serialize` implementation makes, with the
//! names of fields and variants and any maximum lengths. The [`Schema`] trait
//! gives a type's schema as a constant, built at compile time without
//! allocating, and `#[derive(Schema)]` implements the [`Schema`] trait for
//! types.
//!
//! A [`Key`](key::Key) is an 8-byte hash of a list of strings and schemas, such
//! as a method name and the schemas of its request and response. Two programs
//! that compute the same key from their own types can cheaply check that they
//! agree on those types, without exchanging the schemas. A key is not a
//! security boundary: what is received still needs to be validated.
//!
//! ```rust
//! # #[cfg(feature = "derive")] {
//! use postcard_schema::{
//!     key::{Item, Key, Options},
//!     Schema,
//! };
//!
//! #[derive(Schema)]
//! struct Read {
//!     channel: u8,
//! }
//!
//! const READ: Key = Key::new(
//!     &Options::DEFAULT,
//!     &[Item::String("read"), Item::Schema(Read::SCHEMA)],
//! );
//! # }
//! ```
//!
//! With the `alloc` or `std` feature, the `Owned*` types in [`schema`] hold a
//! schema built at runtime, such as one received from a peer, and
//! [`Key::new_owned`](key::Key::new_owned) computes their keys.
//!
//! A schema that disagrees with its type's serde implementations gives wrong
//! keys and wrong decoding. With the `alloc` or `std` feature,
//! [`verify`](verify::verify) checks a type's schema against sample values;
//! and can be used to test hand-written `Schema` impls.
//!
//! The [postcard-schema specification] defines keys, so that other
//! implementations can compute the same keys.
//!
//! # Features
//!
//! | Feature                   | Enables                                                                           |
//! |---                        |---                                                                                |
//! | `derive`                  | `#[derive(Schema)]`                                                               |
//! | `alloc`                   | the `Owned*` schema types, [`verify`], and `Schema` for `alloc` types             |
//! | `std`                     | what `alloc` enables, `Schema` for `std` types, and the owned [`max_len`] types   |
//! | `defmt-v1`                | `defmt::Format` for [`Key`](key::Key)                                             |
//! | `key-v2`                  | version 2 keys, [`Key::new_v2`](key::Key::new_v2), for peers that still use them  |
//! | `postcard-schema-v0_2`    | [`legacy::v0_2`], to read the schemas that postcard-schema 0.2 devices send       |
//! | `postcard-schema-ng-v0_4` | [`legacy::ng_v0_4`], to read the schemas that postcard-schema-ng 0.4 devices send |
//! | `chrono-v0_4`, ...        | `Schema` for that version of that crate's types (see `Cargo.toml`)                |
//!
//! [postcard]: https://docs.rs/postcard
//! [postcard-schema specification]: https://postcard.rs/schema/0.1/

mod impls;
pub mod key;
#[cfg(any(feature = "postcard-schema-v0_2", feature = "postcard-schema-ng-v0_4"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "postcard-schema-v0_2", feature = "postcard-schema-ng-v0_4")))
)]
pub mod legacy;
pub mod schema;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod verify;

/// Types with a fixed upper bound
///
/// The schemas of these types carry the maximum length, which is part of a key
/// when the maximum lengths option
/// ([`Options::max_len`](key::Options::max_len)) is enabled.
pub mod max_len {
    #[cfg(feature = "std")]
    pub use crate::impls::builtins_maxlen::std::{MaxLenBytes, MaxLenString};
    pub use crate::impls::builtins_maxlen::{MaxLenByteSlice, MaxLenStr, TooLong};
}

/// Derive [`Schema`] for a struct or enum
///
/// # Examples
///
/// ```
/// use postcard_schema::Schema;
///
/// #[derive(Schema)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// ```
///
/// # Attributes
///
/// ## `#[serde(rename = "name")]`
///
/// The names of fields, containers, and variants in derived schemas respect `#[serde(rename = "name")]`
/// attributes. Note that this does not include variants like `#[serde(rename(serialize = "ser_name"))]`
/// or `#[serde(rename_all = "...")]`.
///
/// ## `#[postcard(crate = ...)]`
///
/// The `#[postcard(crate = ...)]` attribute can be used to specify a path to the `postcard_schema`
/// crate instance to use when referring to [`Schema`] and [schema types](schema) from generated
/// code. This is normally only applicable when invoking re-exported derives from a different crate.
///
/// ```
/// # use postcard_schema::Schema;
/// use postcard_schema as reexported_postcard_schema;
///
/// #[derive(Schema)]
/// #[postcard(crate = reexported_postcard_schema)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// ```
///
/// ## `#[postcard(bound = ...)]`
///
/// The `#[postcard(bound = ...)]` attribute can be used to overwrite the default bounds when
/// deriving [`Schema`]. The default bounds are `T: Schema` for each type parameter `T`.
///
/// ```
/// # use postcard_schema::Schema;
/// #[derive(Schema)]
/// #[postcard(bound = "")]
/// struct Foo<F: Bar, T: Schema>(F::Wrap<T>);
///
/// trait Bar {
///     type Wrap<T: Schema>: Schema;
/// }
///
/// struct NoSchema;
/// impl Bar for NoSchema {
///     type Wrap<T: Schema> = Option<T>;
/// }
///
/// Foo::<NoSchema, u8>::SCHEMA;
/// ```
#[cfg(feature = "derive")]
pub use postcard_derive::Schema;

/// A trait that represents a compile time calculated schema
pub trait Schema {
    /// A recursive data structure that describes the schema of the given
    /// type.
    const SCHEMA: &'static schema::DataModelType;
}
