//! postcard roundtrip fuzzer: decode -> re-encode -> re-decode -> assert equal.
//!
//! postcard is a binary, non-self-describing format. The deserializer parses
//! attacker-controlled varint length prefixes, string/map/sequence lengths and
//! COBS framing, so arbitrary input is the correct fuzz surface. A recursive
//! self-describing value (`FuzzValue`) drives every type path. There is no
//! recursion limit, so a deeply-nested input can also trigger a stack overflow
//! here — exactly the class of bug OSS-Fuzz is built to surface for Rust.
#![no_main]

use std::collections::BTreeMap;

use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

/// Recursive, self-describing value covering every serde type path postcard
/// must serialize/deserialize (unit, bool, int, uint, float, str, bytes, seq,
/// map).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum FuzzValue {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    Str(String),
    Bytes(Vec<u8>),
    Seq(Vec<FuzzValue>),
    Map(BTreeMap<String, FuzzValue>),
}

fuzz_target!(|data: &[u8]| {
    // 1. Deserialize arbitrary input into a recursive value.
    if let Ok(v) = postcard::from_bytes::<FuzzValue>(data) {
        // 2. Re-encode it.
        if let Ok(bytes) = postcard::to_stdvec(&v) {
            // 3. Decode the canonical bytes again and require stability.
            if let Ok(v2) = postcard::from_bytes::<FuzzValue>(&bytes) {
                assert_eq!(v, v2, "postcard roundtrip produced a different value");
            }
        }
    }
    // Also run the raw input through the fixed-buffer serializer path.
    let mut buf = [0u8; 4096];
    let _ = postcard::to_slice(&FuzzValue::Bytes(data.to_vec()), &mut buf);
});
