#![cfg_attr(not(test), no_std)]

mod de;
mod error;
mod ser;
mod varint;

pub use de::{
    from_bytes,
    from_bytes_cobs,
    take_from_bytes,
    take_from_bytes_cobs,
};
pub use error::{Error, Result};
pub use ser::{
    flavor::{
        SerFlavor,
        HVec,
        Cobs,
        Slice,
    },
    serializer::{
        Serializer,
    },

    serialize_with_flavor,
    to_vec,
    to_vec_cobs,
    to_slice,
    to_slice_cobs,
};
pub use de::{
    Deserializer,
};
