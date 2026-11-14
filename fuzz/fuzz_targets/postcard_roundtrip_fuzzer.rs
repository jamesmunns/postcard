//! postcard roundtrip fuzzer: decode -> re-encode -> re-decode -> assert equal.
//!
//! postcard is a binary, non-self-describing format. The deserializer parses
//! attacker-controlled varint length prefixes, string/map/sequence lengths and
//! COBS framing, so arbitrary input is the correct fuzz surface. A recursive
//! self-describing value (`FuzzValue`) drives every type path. There is no
//! recursion limit, so a deeply-nested input can also trigger a stack overflow
//! here — exactly the class of bug OSS-Fuzz is built to surface for Rust.
#![no_main]

use libfuzzer_sys::fuzz_target;

use crate::common::FuzzValue;

mod common;

/// Assert that `value` encodes and decodes back to the identical value.
fn assert_stable_roundtrip(value: &FuzzValue) {
    // Heap path (varint framing).
    if let Ok(bytes) = postcard::to_stdvec(value) {
        let round = postcard::from_bytes::<FuzzValue>(&bytes);
        assert_eq!(round.as_ref(), Ok(value), "varint roundtrip unstable");
    }

    // Fixed-buffer path (varint framing into a caller-provided slice).
    let mut buf = [0u8; 4096];
    if let Ok(written) = postcard::to_slice(value, &mut buf) {
        let round = postcard::from_bytes::<FuzzValue>(written);
        assert_eq!(round.as_ref(), Ok(value), "slice roundtrip unstable");
    }

    // COBS-framing roundtrip (the crate's second framing scheme).
    let mut out = [0u8; 8192];
    let mut in_buf = [0u8; 8192];
    if let Ok(written) = postcard::to_slice_cobs(value, &mut out) {
        in_buf[..written.len()].copy_from_slice(written);
        let round = postcard::from_bytes_cobs::<FuzzValue>(&mut in_buf[..written.len()]);
        assert_eq!(round.as_ref(), Ok(value), "COBS roundtrip unstable");
    }
}

fuzz_target!(|data: &[u8]| {
    // Deserialize arbitrary input into a recursive value, then prove the
    // encoder and decoder are mutually stable across all three framing paths.
    if let Ok(v) = postcard::from_bytes::<FuzzValue>(data) {
        assert_stable_roundtrip(&v);
    }
});
