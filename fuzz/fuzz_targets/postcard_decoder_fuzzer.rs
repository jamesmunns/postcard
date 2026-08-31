//! postcard decoder fuzzer: pure deserialization coverage over arbitrary
//! (mostly malformed) bytes. Unlike the roundtrip fuzzer, this still reaches
//! deep into the decoder when the input never decodes cleanly.
#![no_main]

use std::collections::{BTreeMap, HashMap};

use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

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
    // Deep recursive value — catches stack overflow on nesting bombs.
    let _ = postcard::from_bytes::<FuzzValue>(data);

    // Common real-world payload shapes to broaden code-path coverage.
    let _ = postcard::from_bytes::<Vec<(u32, String)>>(data);
    let _ = postcard::from_bytes::<HashMap<u16, Vec<u8>>>(data);
    let _ = postcard::from_bytes::<Option<(String, [u8; 8], i64)>>(data);
    let _ = postcard::from_bytes::<Vec<Vec<Vec<u8>>>>(data);

    // COBS-flavored framing (alternative decode path in the same crate).
    // NB: `from_bytes_cobs` consumes a mutable slice.
    let mut cobs_buf = data.to_vec();
    let _ = postcard::from_bytes_cobs::<FuzzValue>(&mut cobs_buf);
});
