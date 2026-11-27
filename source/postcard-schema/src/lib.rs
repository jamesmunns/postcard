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
//! A [`Key`] is an 8-byte hash of a list of strings and schemas, such
//! as a method name and the schemas of its request and response. Two programs
//! that compute the same key from their own types can cheaply check that they
//! agree on those types, without exchanging the schemas. A key is not a
//! security boundary: what is received still needs to be validated.
//!
//! ```rust
//! # #[cfg(feature = "derive")] {
//! use postcard_schema::{Item, Key, Options, Schema};
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
//! [`Key::new_owned`] computes their keys.
//!
//! Schemas are sent as a [schema table](table): an ordinary type,
//! holding one or more schemas with each part stored once. [`schema_table!`]
//! builds a schema table at compile time, and [`schema_table_bytes!`]
//! builds its postcard encoding at compile time. With the `alloc` or `std`
//! feature, [`OwnedSchemaTable`] can be used
//! to build or receive one at runtime, and `decode` converts a table back into
//! schemas.
//!
//! ```rust
//! # #[cfg(all(feature = "derive", feature = "std"))] {
//! use postcard_schema::{schema_table, OwnedSchemaTable, Schema};
//!
//! #[derive(Schema)]
//! struct Read {
//!     channel: u8,
//! }
//!
//! // Built at compile time, and sent as postcard data
//! let bytes = postcard::to_vec(&schema_table!(Read, Option<Read>)).unwrap();
//!
//! // Received, then checked and expanded, within the default limits
//! let table: OwnedSchemaTable = postcard::from_bytes(&bytes).unwrap();
//! let schemas = table.decode().unwrap();
//! assert_eq!(schemas[0], Read::SCHEMA.into());
//! # }
//! ```
//!
//! A schema that disagrees with its type's serde implementations calculates
//! incorrect keys and performs incorrect decoding. With the `alloc` or `std`
//! feature, [`verify`](verify::verify) checks a type's schema against sample
//! values, and can be used to test hand-written `Schema` impls.
//!
//! The [postcard-schema specification] defines keys and schema tables, so
//! that other implementations can compute the same keys and read the same
//! tables.
//!
//! # Features
//!
//! | Feature            | Enables                                                                         |
//! |---                 |---                                                                              |
//! | `derive`           | `#[derive(Schema)]`                                                             |
//! | `alloc`            | the `Owned*` schema types, [`verify`], and `Schema` for `alloc` types           |
//! | `std`              | what `alloc` enables, `Schema` for `std` types, and the owned [`max_len`] types |
//! | `defmt-v1`         | `defmt::Format` for [`Key`]                                                     |
//! | `chrono-v0_4`, ... | `Schema` for that version of that crate's types (see `Cargo.toml`)              |
//!
//! # Talking to older peers
//!
//! The [`legacy`] module, behind its own features, is only for talking to peers
//! built with an older version. New code doesn't need it.
//!
//! | Feature                 | Enables                                                                           |
//! |---                      |---                                                                                |
//! | `legacy-key-v2`         | [`legacy::key_v2`], to compute the version 2 keys that older peers use            |
//! | `legacy-schema-v0_2`    | [`legacy::v0_2`], to read the schemas that postcard-schema 0.2 devices send       |
//! | `legacy-schema-ng-v0_4` | [`legacy::ng_v0_4`], to read the schemas that postcard-schema-ng 0.4 devices send |
//!
//! [postcard]: https://docs.rs/postcard
//! [postcard-schema specification]: https://postcard.rs/schema/0.1/

mod impls;
pub mod key;
#[cfg(any(
    feature = "legacy-key-v2",
    feature = "legacy-schema-v0_2",
    feature = "legacy-schema-ng-v0_4"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "legacy-key-v2",
        feature = "legacy-schema-v0_2",
        feature = "legacy-schema-ng-v0_4"
    )))
)]
pub mod legacy;
pub mod schema;
pub mod table;

pub use key::{Item, Key, Options};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use table::OwnedSchemaTable;
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
/// ## `#[serde(with = "serde_bytes")]`
///
/// Fields marked `#[serde(with = "serde_bytes")]` have a
/// [`ByteArray`](schema::DataModelType::ByteArray) schema, as `serde_bytes`
/// serializes them as bytes. This works for `[u8]`, `[u8; N]`, `Vec<u8>`, and
/// references, `Box`es, `Cow`s and `Option`s of these.
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

/// Not public API: used by `#[derive(Schema)]`
#[doc(hidden)]
pub mod __private {
    use crate::schema::DataModelType;

    /// The schema of a field marked `#[serde(with = "serde_bytes")]`
    ///
    /// Implemented for the types `serde_bytes` serializes as bytes.
    pub trait SerdeBytes {
        const SCHEMA: &'static DataModelType;
    }

    impl SerdeBytes for [u8] {
        const SCHEMA: &'static DataModelType = &DataModelType::ByteArray { max_len: None };
    }

    impl<const N: usize> SerdeBytes for [u8; N] {
        const SCHEMA: &'static DataModelType = &DataModelType::ByteArray { max_len: Some(N) };
    }

    impl<T: SerdeBytes + ?Sized> SerdeBytes for &T {
        const SCHEMA: &'static DataModelType = T::SCHEMA;
    }

    impl<T: SerdeBytes + ?Sized> SerdeBytes for &mut T {
        const SCHEMA: &'static DataModelType = T::SCHEMA;
    }

    impl<T: SerdeBytes> SerdeBytes for Option<T> {
        const SCHEMA: &'static DataModelType = &DataModelType::Option(T::SCHEMA);
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    mod alloc_impls {
        extern crate alloc;
        use super::SerdeBytes;
        use crate::schema::DataModelType;
        use alloc::{
            borrow::{Cow, ToOwned},
            boxed::Box,
            vec::Vec,
        };

        impl SerdeBytes for Vec<u8> {
            const SCHEMA: &'static DataModelType = &DataModelType::ByteArray { max_len: None };
        }

        impl<T: SerdeBytes + ?Sized> SerdeBytes for Box<T> {
            const SCHEMA: &'static DataModelType = T::SCHEMA;
        }

        impl<T: SerdeBytes + ToOwned + ?Sized> SerdeBytes for Cow<'_, T> {
            const SCHEMA: &'static DataModelType = T::SCHEMA;
        }
    }
}

/// A trait that represents a compile time calculated schema
pub trait Schema {
    /// A recursive data structure that describes the schema of the given
    /// type.
    const SCHEMA: &'static schema::DataModelType;
}
