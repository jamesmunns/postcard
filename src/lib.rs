#![cfg_attr(not(any(test, feature = "use-std")), no_std)]
#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

pub mod accumulator;
mod de;
mod error;
mod ser;
mod varint;

// Still experimental! Don't make pub pub.
pub(crate) mod max_size;
pub(crate) mod schema;

/// # Experimental Postcard Features
///
/// Items inside this module require various feature flags, and are not
/// subject to SemVer stability. Items may be removed or deprecated at
/// any point.
///
/// ## Derive
///
/// The `experimental-derive` feature enables two experimental features:
///
/// * Max size calculation
/// * Message schema generation
///
/// ### Max Size Calculation
///
/// This features enables calculation of the Max serialized size of a message as
/// an associated `usize` constant called `POSTCARD_MAX_SIZE`. It also provides a
/// `#[derive(MaxSize)]` macro that can be used for calculating user types.
///
/// This is useful for determining the maximum buffer size needed when recieving
/// or sending a message that has been serialized.
///
/// NOTE: This only covers the size of "plain" flavored messages, e.g. not with COBS
/// or any other Flavors applied. The overhead for these flavors must be calculated
/// separately.
///
/// Please report any missing types, or any incorrectly calculated values.
///
/// ### Message Schema Generation
///
/// This feature enables the generation of a schema of a given message at compile
/// time. At the moment, this is only exposed as a [`NamedType`](crate::experimental::schema::NamedType)
/// which is a recursive data structure describing the schema. In the future, it is planned
/// to provide formatting functions that emit this as a human or machine readable schema.
///
/// NOTE: This only covers the schema of "plain" flavored messages, e.g. not with COBS
/// or any other Flavors applied. The format of these flavors must be calculated
/// separately.
///
/// Please report any missing types, or any incorrectly calculated schemas.
pub mod experimental {
    /// Compile time max-serialization size calculation
    #[cfg(feature = "experimental-derive")]
    pub mod max_size {
        // NOTE: This is the trait...
        pub use crate::max_size::MaxSize;
        // NOTE: ...and this is the derive macro
        pub use postcard_derive::MaxSize;
    }

    /// Compile time Schema generation
    #[cfg(feature = "experimental-derive")]
    pub mod schema {
        // NOTE: This is the trait...
        pub use crate::schema::{NamedType, NamedValue, NamedVariant, Schema, SdmTy, Varint};
        // NOTE: ...and this is the derive macro
        pub use postcard_derive::Schema;
    }
}

pub use de::deserializer::Deserializer;
pub use de::flavors as de_flavors;
pub use de::{from_bytes, from_bytes_cobs, take_from_bytes, take_from_bytes_cobs};
pub use error::{Error, Result};
pub use ser::flavors as ser_flavors;
pub use ser::{serialize_with_flavor, serializer::Serializer, to_slice, to_slice_cobs};

#[cfg(feature = "heapless")]
pub use ser::{to_vec, to_vec_cobs};

#[cfg(feature = "use-std")]
pub use ser::{to_stdvec, to_stdvec_cobs};

#[cfg(feature = "alloc")]
pub use ser::{to_allocvec, to_allocvec_cobs};

#[cfg(test)]
mod test {
    #[test]
    fn varint_boundary_canon() {
        let x = u32::MAX;
        let mut buf = [0u8; 5];
        let used = crate::to_slice(&x, &mut buf).unwrap();
        let deser: u32 = crate::from_bytes(used).unwrap();
        assert_eq!(deser, u32::MAX);
        assert_eq!(used, &mut [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        let deser: Result<u32, crate::Error> = crate::from_bytes(&[0xFF, 0xFF, 0xFF, 0xFF, 0x1F]);
        assert_eq!(deser, Err(crate::Error::DeserializeBadVarint));
    }

    #[test]
    fn signed_int128() {
        let x = -19490127978232325886905073712831_i128;
        let mut buf = [0u8; 32];
        let used = crate::to_slice(&x, &mut buf).unwrap();
        let deser: i128 = crate::from_bytes(used).unwrap();
        assert_eq!(deser, x);
    }
}
