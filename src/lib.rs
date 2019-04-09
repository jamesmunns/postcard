// #![cfg_attr(not(test), no_std)]

mod de;
mod error;
mod ser;
mod varint;

pub use de::from_bytes;
pub use error::{Error, Result};
pub use ser::{to_vec, to_vec_cobs};
