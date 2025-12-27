#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod de;

pub mod fixint;
mod ser;
/// todo: keep public?
pub mod varint;

/// # Experimental Postcard Features
///
/// Items inside this module require various feature flags, and are not
/// subject to SemVer stability. Items may be removed or deprecated at
/// any point.
pub mod experimental {
    pub use crate::ser::serialized_size;
}

pub use de::deserializer::Deserializer;
pub use de::flavors as de_flavors;
pub use de::{from_bytes, take_from_bytes};
pub use ser::flavors as ser_flavors;
pub use ser::{serialize_with_flavor, serializer::Serializer, to_extend, to_slice};

#[cfg(feature = "std")]
pub use ser::to_io;

#[cfg(feature = "std")]
pub use de::from_io;

#[cfg(any(feature = "alloc", feature = "std"))]
pub use ser::to_vec;

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
        let deser: Result<u32, crate::de::deserializer::DeserializerError> =
            crate::from_bytes(&[0xFF, 0xFF, 0xFF, 0xFF, 0x1F]);
        assert_eq!(
            deser,
            Err(crate::de::deserializer::DeserializerError::BadVarint)
        );
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
