//! Postcard Core
#![cfg_attr(not(any(test, feature = "std")), no_std)]

pub mod de;
pub mod ser;
pub mod varint;
