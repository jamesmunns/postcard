#![cfg_attr(not(test), no_std)]

mod error;
mod ser;
mod varint;
mod de;

pub use ser::to_vec;
pub use varint::{
    VarintUsize,
    VarintBuf,
};
