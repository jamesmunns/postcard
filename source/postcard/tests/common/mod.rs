//! Helpers shared by the postcard spec tests.
#![allow(dead_code)]

use core::fmt::Debug;
use serde::{Serialize, de::DeserializeOwned};

/// Serialize `val`, returning the encoded bytes.
pub fn encode<T: Serialize + ?Sized>(val: &T) -> Vec<u8> {
    let mut out = Vec::new();
    postcard::to_extend(val, &mut out).unwrap();
    out
}

/// Ensure `val` encodes to exactly `expected`, and that `expected` decodes
/// back to `val` with no bytes left over.
pub fn roundtrip<T>(val: T, expected: &[u8])
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    assert_eq!(encode(&val), expected, "encoding of {val:?}");
    let (out, rest) = postcard::take_from_bytes::<T>(expected).unwrap();
    assert_eq!(out, val, "decoding of {expected:?}");
    assert_eq!(rest, [].as_slice(), "trailing bytes for {val:?}");
}
