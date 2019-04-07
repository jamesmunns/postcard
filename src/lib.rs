// #![cfg_attr(not(test), no_std)]

mod de;
mod error;
mod ser;
mod varint;

pub use de::from_bytes;
pub use ser::to_vec;
