#![cfg_attr(not(test), no_std)]

pub mod error;
pub mod ser;
pub mod varint;

pub use ser::to_vec;
pub use varint::{
    VarintUsize,
};
