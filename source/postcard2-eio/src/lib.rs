#![cfg_attr(not(test), no_std)]

//! [`embedded-io`](https://docs.rs/embedded-io) support for [`postcard2`]

#[cfg(feature = "embedded-io-v0_7")]
pub mod v0_7;
