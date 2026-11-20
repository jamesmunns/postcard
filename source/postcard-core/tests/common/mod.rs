//! Helpers shared by the postcard-core integration tests.
#![allow(dead_code, unused_macros)]

use postcard_core::{de, ser};

/// Minimal flavor so tests depend only on `postcard-core`.
#[derive(Default)]
pub struct VecFlavor(pub Vec<u8>);

impl ser::Flavor for VecFlavor {
    type Output = Vec<u8>;
    type PushError = core::convert::Infallible;
    type FinalizeError = core::convert::Infallible;

    fn try_push(&mut self, data: u8) -> Result<(), Self::PushError> {
        self.0.push(data);
        Ok(())
    }

    fn finalize(self) -> Result<Self::Output, Self::FinalizeError> {
        Ok(self.0)
    }
}

pub fn encode(f: impl FnOnce(&mut VecFlavor) -> Result<(), core::convert::Infallible>) -> Vec<u8> {
    let mut fl = VecFlavor::default();
    f(&mut fl).unwrap();
    ser::Flavor::finalize(fl).unwrap()
}

pub fn slice(bytes: &[u8]) -> de::flavors::Slice<'_> {
    de::flavors::Slice::new(bytes)
}

/// Round trip `$val`, ensuring it matches `$decoded`, and there are no excess
/// encoded bytes. Returns the encoded bytes.
macro_rules! roundtrip {
    ($push:path, $take:path, $val:expr, $decoded:expr) => {{
        let val = $val;
        let bytes = encode(|f| $push(f, val));
        {
            let mut fl = slice(&bytes);
            let out = $take(&mut fl).unwrap();
            assert_eq!(out, $decoded, "decoding of {:?}", val);
            let rest = de::Flavor::finalize(fl).unwrap();
            assert_eq!(rest, [].as_slice(), "trailing bytes for {:?}", val);
        }
        bytes
    }};
}
