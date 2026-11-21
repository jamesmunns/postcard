use postcard_core::de::DecodeError;
use postcard_core::{de, ser};

#[macro_use]
mod common;
use common::*;

/// Ensure "end of stream" flavor errors cause a decode failure.
#[test]
fn varint_unexpected_end() {
    assert_eq!(
        de::try_take_u32(&mut slice(&[])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
    assert_eq!(
        de::try_take_u32(&mut slice(&[0x80, 0x80])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
}

/// Smoke test the `Slice` deser flavor.
#[test]
fn de_slice() {
    use de::Flavor;
    let data = [1u8, 2, 3, 4, 5];
    let mut fl = slice(&data);
    assert_eq!(fl.size_hint(), Some(5));
    assert_eq!(fl.pop(), Ok(1));
    assert_eq!(fl.try_take_n(2), Ok([2, 3].as_slice()));
    assert_eq!(fl.size_hint(), Some(2));
    assert_eq!(fl.try_take_n(3), Err(de::UnexpectedEnd));
    assert_eq!(fl.try_take_n(1), Ok([4].as_slice()));
    assert_eq!(fl.finalize(), Ok([5].as_slice()));

    let mut fl = slice(&[]);
    assert_eq!(fl.pop(), Err(de::UnexpectedEnd));
    assert_eq!(fl.try_take_n(0), Ok([].as_slice()));
}

/// Smoke test the `Slice` ser flavor.
#[test]
fn ser_slice() {
    use ser::Flavor;
    let mut buf = [0u8; 4];
    let mut fl = ser::flavors::Slice::new(&mut buf);
    assert_eq!(fl.try_push(1), Ok(()));
    assert_eq!(fl.try_extend(&[2, 3]), Ok(()));
    assert_eq!(fl.try_extend(&[4, 5]), Err(ser::BufferFull)); // writes neither byte
    assert_eq!(fl.try_push(4), Ok(()));
    assert_eq!(fl.try_push(5), Err(ser::BufferFull));
    assert_eq!(fl.try_extend(&[]), Ok(()));
    assert_eq!(fl.finalize().unwrap(), &[1, 2, 3, 4]);

    // Only the bytes written are returned
    let mut buf = [0u8; 4];
    let mut fl = ser::flavors::Slice::new(&mut buf);
    fl.try_push(9).unwrap();
    assert_eq!(fl.finalize().unwrap(), &[9]);

    // A zero-length buffer
    let mut fl = ser::flavors::Slice::new(&mut []);
    assert_eq!(fl.try_push(1), Err(ser::BufferFull));
    assert_eq!(fl.finalize().unwrap(), [].as_slice());
}

// Ensure that the current interface is suitable for encoding a COBS flavor.
mod cobs {
    use postcard_core::ser::{self, Flavor, Patch};

    pub struct Cobs<P: Patch> {
        inner: P,
        code_pos: Option<usize>,
        run: u8,
    }

    impl<P: Patch> Cobs<P> {
        pub fn try_new(inner: P) -> Result<Self, P::PushError> {
            let mut me = Cobs {
                inner,
                code_pos: None,
                run: 0,
            };
            me.start_block()?;
            Ok(me)
        }

        fn start_block(&mut self) -> Result<(), P::PushError> {
            self.code_pos = Some(self.inner.written_mut().len());
            self.run = 1;
            self.inner.try_push(0)
        }

        fn end_block(&mut self) {
            if let Some(pos) = self.code_pos.take() {
                self.inner.written_mut()[pos] = self.run;
            }
        }
    }

    impl<P: Patch> Flavor for Cobs<P> {
        type Output = P::Output;
        type PushError = P::PushError;
        type FinalizeError = P::FinalizeError;

        fn try_push(&mut self, data: u8) -> Result<(), Self::PushError> {
            if self.code_pos.is_none() {
                self.start_block()?;
            }
            if data == 0 {
                self.end_block();
                self.start_block()
            } else {
                self.inner.try_push(data)?;
                self.run += 1;
                if self.run == 0xFF {
                    self.end_block();
                }
                Ok(())
            }
        }

        fn finalize(mut self) -> Result<Self::Output, Self::FinalizeError> {
            self.end_block();
            self.inner.try_push(0).ok().unwrap();
            self.inner.finalize()
        }
    }

    pub fn encode(prefix: &[u8], data: &[u8]) -> Vec<u8> {
        let mut buf = vec![0u8; prefix.len() + data.len() + data.len() / 254 + 2];
        let mut slice = ser::flavors::Slice::new(&mut buf);
        slice.try_extend(prefix).unwrap();
        let mut c = Cobs::try_new(slice).unwrap();
        c.try_extend(data).unwrap();
        c.finalize().unwrap().to_vec()
    }
}

/// Ensure the `Patch` trait is suitable for implementing a `Cobs` flavor.
#[test]
fn patch_cobs_vectors() {
    let range = |a: u8, b: u8| (a..=b).collect::<Vec<u8>>();
    let concat = |parts: &[&[u8]]| parts.concat();

    // Test cases from wikipedia
    let cases: Vec<(Vec<u8>, Vec<u8>)> = vec![
        (vec![], vec![0x01, 0x00]),
        (vec![0x00], vec![0x01, 0x01, 0x00]),
        (vec![0x00, 0x00], vec![0x01, 0x01, 0x01, 0x00]),
        (vec![0x00, 0x11, 0x00], vec![0x01, 0x02, 0x11, 0x01, 0x00]),
        (
            vec![0x11, 0x22, 0x00, 0x33],
            vec![0x03, 0x11, 0x22, 0x02, 0x33, 0x00],
        ),
        (
            vec![0x11, 0x22, 0x33, 0x44],
            vec![0x05, 0x11, 0x22, 0x33, 0x44, 0x00],
        ),
        (
            vec![0x11, 0x00, 0x00, 0x00],
            vec![0x02, 0x11, 0x01, 0x01, 0x01, 0x00],
        ),
        // 254 non-zero bytes: exactly one full block
        (
            range(0x01, 0xFE),
            concat(&[&[0xFF], &range(0x01, 0xFE), &[0x00]]),
        ),
        // A zero, then a full block
        (
            concat(&[&[0x00], &range(0x01, 0xFE)]),
            concat(&[&[0x01, 0xFF], &range(0x01, 0xFE), &[0x00]]),
        ),
        // 255 non-zero bytes: the block splits after 254
        (
            range(0x01, 0xFF),
            concat(&[&[0xFF], &range(0x01, 0xFE), &[0x02, 0xFF, 0x00]]),
        ),
        // A full block, then a zero
        (
            concat(&[&range(0x02, 0xFF), &[0x00]]),
            concat(&[&[0xFF], &range(0x02, 0xFF), &[0x01, 0x01, 0x00]]),
        ),
        // 253 non-zero bytes, a zero, then one more byte
        (
            concat(&[&range(0x03, 0xFF), &[0x00, 0x01]]),
            concat(&[&[0xFE], &range(0x03, 0xFF), &[0x02, 0x01, 0x00]]),
        ),
    ];
    for (data, expected) in cases {
        assert_eq!(cobs::encode(&[], &data), expected, "encoding {data:02X?}");
    }
}

/// Ensure the `Patch` impl does not include unpushed buffer contents.
#[test]
fn patch_slice() {
    use ser::{Flavor, Patch};
    let mut buf = [0u8; 4];
    let mut fl = ser::flavors::Slice::new(&mut buf);
    assert_eq!(fl.written_mut(), [].as_slice());
    fl.try_extend(&[1, 2, 3]).unwrap();
    assert_eq!(fl.written_mut(), &[1, 2, 3]);
    fl.written_mut()[1] = 9;
    fl.try_push(4).unwrap();
    assert_eq!(fl.finalize().unwrap(), &[1, 9, 3, 4]);
}

/// Ensure a `Patch` user that records its start position leaves earlier data
/// untouched, using the cobs impl
#[test]
fn patch_cobs_after_existing_data() {
    let out = cobs::encode(&[0xAA, 0xBB], &[0x11, 0x00, 0x22]);
    assert_eq!(out, [0xAA, 0xBB, 0x02, 0x11, 0x02, 0x22, 0x00]);
}

/// Smoke test `ExtendFlavor` ser flavor
#[test]
fn extend_flavor() {
    use ser::Flavor;
    let mut fl = ser::flavors::ExtendFlavor::new(vec![0xAA]);
    ser::try_push_u16(&mut fl, 300).unwrap();
    fl.try_push(1).unwrap();
    assert_eq!(fl.finalize().unwrap(), [0xAA, 0xAC, 0x02, 0x01]);
}

// A helper flavor that borrows another flavor for less than `'de`, so callers can
// decode a few items through a wrapper and then keep using the inner flavor.
mod by_ref {
    use postcard_core::de::Flavor;

    pub struct ByRef<'a, F>(pub &'a mut F);

    impl<'de, F: Flavor<'de>> Flavor<'de> for ByRef<'_, F> {
        type Remainder = ();
        type PopError = F::PopError;
        type FinalizeError = core::convert::Infallible;

        fn pop(&mut self) -> Result<u8, Self::PopError> {
            self.0.pop()
        }

        fn size_hint(&self) -> Option<usize> {
            self.0.size_hint()
        }

        fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8], Self::PopError> {
            self.0.try_take_n(ct)
        }

        fn finalize(self) -> Result<(), Self::FinalizeError> {
            Ok(())
        }
    }
}

/// This test exists to prove to myself we don't need a `Flavor<'de>: 'de` bound.
#[test]
fn de_wrapper_borrowing_inner() {
    use de::Flavor;
    let data = [0x02, b'h', b'i', 0x07, 0x09];
    let mut inner = slice(&data);
    let s = {
        let mut wrapper = by_ref::ByRef(&mut inner);
        let s = de::try_take_str(&mut wrapper).unwrap();
        assert_eq!(de::try_take_u8(&mut wrapper), Ok(0x07));
        s
    };
    assert_eq!(de::try_take_u8(&mut inner), Ok(0x09));
    assert_eq!(inner.finalize(), Ok([].as_slice()));
    assert_eq!(s, "hi");
}

/// Ensure various bad encodings are reported correctly.
#[test]
fn decode_error_kinds() {
    // A length longer than any varint(usize)
    let bad_len = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ];
    assert_eq!(
        de::try_take_str(&mut slice(&bad_len)),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        de::try_take_bytes(&mut slice(&bad_len)),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        de::try_take_char(&mut slice(&bad_len)),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        // length 2, one byte
        de::try_take_str(&mut slice(&[0x02, b'h'])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
    assert_eq!(
        de::try_take_str(&mut slice(&[0x01, 0xFF])),
        Err(DecodeError::BadUtf8)
    );
    assert_eq!(
        de::try_take_bool(&mut slice(&[])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
    assert_eq!(
        de::try_take_f32(&mut slice(&[0x00, 0x00, 0x00])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
    assert_eq!(
        de::try_take_bytes(&mut slice(&[0x03, 0x01])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
}

/// Ensure the contract of `&mut` flavor impls are correct.
#[test]
fn ser_by_mut_ref() {
    use ser::Flavor;
    let mut buf = [0u8; 8];
    let mut fl = ser::flavors::Slice::new(&mut buf);
    {
        let mut r = &mut fl;
        ser::try_push_u16(&mut r, 300).unwrap();
        r.finalize().unwrap();
    }
    ser::try_push_bool(&mut &mut fl, true).unwrap();
    assert_eq!(fl.try_push(7), Ok(()));
    assert_eq!(fl.finalize().unwrap(), &[0xAC, 0x02, 0x01, 0x07]);
}

/// Ensure `Patch` works as expected through an `&mut` flavor.
#[test]
fn patch_by_mut_ref() {
    use ser::Flavor;
    let mut buf = [0u8; 8];
    let mut fl = ser::flavors::Slice::new(&mut buf);
    fl.try_push(0xAA).unwrap();
    {
        let mut c = cobs::Cobs::try_new(&mut fl).unwrap();
        c.try_extend(&[0x11, 0x00]).unwrap();
        c.finalize().unwrap();
    }
    fl.try_push(0xBB).unwrap();
    assert_eq!(
        fl.finalize().unwrap(),
        &[0xAA, 0x02, 0x11, 0x01, 0x00, 0xBB]
    );
}

/// Ensure `Error::source()` impl works as expected
#[test]
fn decode_error_source() {
    use core::error::Error;
    let e: DecodeError<de::UnexpectedEnd> = DecodeError::Pop(de::UnexpectedEnd);
    assert_eq!(e.to_string(), "Pop");
    assert_eq!(e.source().unwrap().to_string(), "UnexpectedEnd");
    let e: DecodeError<de::UnexpectedEnd> = DecodeError::BadVarint;
    assert_eq!(e.to_string(), "BadVarint");
    assert!(e.source().is_none());
}

/// Ensure `defmt-v1` support is complete using a compile time check.
#[cfg(feature = "defmt-v1")]
#[test]
fn defmt_impls() {
    fn is_format<T: defmt::Format>() {}
    is_format::<ser::BufferFull>();
    is_format::<de::UnexpectedEnd>();
    is_format::<DecodeError<de::UnexpectedEnd>>();
    is_format::<DecodeError<core::convert::Infallible>>();
}

/// Smoke test `Vec` flavor.
#[cfg(feature = "alloc")]
#[test]
fn vec_flavor() {
    use ser::{Flavor, Patch};
    let mut v = Vec::new();
    ser::try_push_u16(&mut v, 300).unwrap();
    v.try_extend(&[1, 2]).unwrap();
    assert_eq!(v.written_mut(), &[0xAC, 0x02, 1, 2]);
    assert_eq!(v.finalize().unwrap(), [0xAC, 0x02, 1, 2]);

    let mut existing = vec![0xAA];
    ser::try_push_bool(&mut &mut existing, true).unwrap();
    assert_eq!(existing, [0xAA, 0x01]);

    let mut c = cobs::Cobs::try_new(&mut existing).unwrap();
    c.try_extend(&[0x11, 0x00]).unwrap();
    c.finalize().unwrap();
    assert_eq!(existing, [0xAA, 0x01, 0x02, 0x11, 0x01, 0x00]);
}

#[test]
fn option_none() {
    assert_eq!(encode(ser::try_push_option_none), [0x00]);
    assert_eq!(de::try_take_option_discrim(&mut slice(&[0x00])), Ok(false));
}

#[test]
fn option_some() {
    assert_eq!(encode(ser::try_push_option_some), [0x01]);
    assert_eq!(de::try_take_option_discrim(&mut slice(&[0x01])), Ok(true));
}

#[test]
fn option_rejects_invalid() {
    for i in 2..=u8::MAX {
        assert_eq!(
            de::try_take_option_discrim(&mut slice(&[i])),
            Err(DecodeError::BadOption)
        );
    }
}
