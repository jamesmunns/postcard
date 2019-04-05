#![cfg_attr(not(test), no_std)]

pub mod error;
pub mod ser;

pub use ser::to_vec;
