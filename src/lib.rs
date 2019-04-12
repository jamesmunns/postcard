#![cfg_attr(not(any(test, feature = "use-std")), no_std)]

mod de;
mod error;
mod ser;
mod varint;

pub use de::deserializer::Deserializer;
pub use de::{deserializer::from_bytes, from_bytes_cobs, take_from_bytes, take_from_bytes_cobs};
pub use error::{Error, Result};
pub use ser::{
    flavors, serialize_with_flavor, serializer::Serializer, to_slice, to_slice_cobs, to_vec,
    to_vec_cobs,
};
