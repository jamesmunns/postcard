#![cfg_attr(not(test), no_std)]

pub struct Postcard {
    _internal: (),
}

pub mod error;
pub mod ser;
