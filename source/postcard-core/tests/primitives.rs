use postcard_core::de::DecodeError;
use postcard_core::{de, ser};

/// Minimal flavor so tests depend only on `postcard-core`.
#[derive(Default)]
struct VecFlavor(Vec<u8>);

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

fn encode(f: impl FnOnce(&mut VecFlavor) -> Result<(), core::convert::Infallible>) -> Vec<u8> {
    let mut fl = VecFlavor::default();
    f(&mut fl).unwrap();
    ser::Flavor::finalize(fl).unwrap()
}

fn slice(bytes: &[u8]) -> de::flavors::Slice<'_> {
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
            assert_eq!(rest, &[] as &[u8], "trailing bytes for {:?}", val);
        }
        bytes
    }};
}

/// Round trip `bool`, ensure that invalid encodings are rejected.
#[test]
fn bool() {
    let b = roundtrip!(ser::try_push_bool, de::try_take_bool, false, false);
    assert_eq!(b, [0x00]);
    let b = roundtrip!(ser::try_push_bool, de::try_take_bool, true, true);
    assert_eq!(b, [0x01]);
    for i in 2..=u8::MAX {
        assert_eq!(
            de::try_take_bool(&mut slice(&[i])),
            Err(DecodeError::BadBool)
        );
    }
    assert_eq!(
        de::try_take_bool(&mut slice(&[])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
}

/// Ensure `u8` and `i8` round trip across all values.
#[test]
fn eight_bit() {
    for v in 0..=u8::MAX {
        let b = roundtrip!(ser::try_push_u8, de::try_take_u8, v, v);
        assert_eq!(b, [v]);
    }
    for v in i8::MIN..=i8::MAX {
        let b = roundtrip!(ser::try_push_i8, de::try_take_i8, v, v);
        assert_eq!(b, [v as u8]);
    }
}

/// Ensure `u16` and `i16` produce exactly the bytes listed in the "Unsigned Integer
/// Encoding" and "Signed Integer Encoding" tables of the wire format spec, and round trip
/// back to the same values.
#[test]
fn spec_vectors() {
    let u16s: &[(u16, &[u8])] = &[
        (0, &[0x00]),
        (127, &[0x7F]),
        (128, &[0x80, 0x01]),
        (16383, &[0xFF, 0x7F]),
        (16384, &[0x80, 0x80, 0x01]),
        (16385, &[0x81, 0x80, 0x01]),
        (65535, &[0xFF, 0xFF, 0x03]),
    ];
    for &(v, expected) in u16s {
        let b = roundtrip!(ser::try_push_u16, de::try_take_u16, v, v);
        assert_eq!(b, expected, "encoding of {v}");
    }

    let i16s: &[(i16, &[u8])] = &[
        (0, &[0x00]),
        (-1, &[0x01]),
        (1, &[0x02]),
        (63, &[0x7E]),
        (-64, &[0x7F]),
        (64, &[0x80, 0x01]),
        (-65, &[0x81, 0x01]),
        (32767, &[0xFE, 0xFF, 0x03]),
        (-32768, &[0xFF, 0xFF, 0x03]),
    ];
    for &(v, expected) in i16s {
        let b = roundtrip!(ser::try_push_i16, de::try_take_i16, v, v);
        assert_eq!(b, expected, "encoding of {v}");
    }
}

macro_rules! varint_bounds {
    ($name:ident, $ty:ty, $push:path, $take:path) => {
        /// Ensure that the given varint encoded integer type:
        ///
        /// * round-trips its minimum, maximum, and values around the one and two byte boundaries
        /// * never uses more than `varint_max` bytes
        /// * is encoded canonically, with no trailing zero byte
        #[test]
        fn $name() {
            let max_bytes = postcard_core::varint::varint_max::<$ty>();
            for v in [
                <$ty>::MIN,
                <$ty>::MIN + 1,
                0,
                1,
                63,
                64,
                127,
                128,
                <$ty>::MAX - 1,
                <$ty>::MAX,
            ] {
                let bytes = roundtrip!($push, $take, v, v);
                assert!(bytes.len() <= max_bytes);
                assert!(bytes.len() == 1 || bytes[bytes.len() - 1] != 0);
            }
        }
    };
}

varint_bounds!(u16_bounds, u16, ser::try_push_u16, de::try_take_u16);
varint_bounds!(u32_bounds, u32, ser::try_push_u32, de::try_take_u32);
varint_bounds!(u64_bounds, u64, ser::try_push_u64, de::try_take_u64);
varint_bounds!(u128_bounds, u128, ser::try_push_u128, de::try_take_u128);
varint_bounds!(usize_bounds, usize, ser::try_push_usize, de::try_take_usize);
varint_bounds!(i16_bounds, i16, ser::try_push_i16, de::try_take_i16);
varint_bounds!(i32_bounds, i32, ser::try_push_i32, de::try_take_i32);
varint_bounds!(i64_bounds, i64, ser::try_push_i64, de::try_take_i64);
varint_bounds!(i128_bounds, i128, ser::try_push_i128, de::try_take_i128);
varint_bounds!(isize_bounds, isize, ser::try_push_isize, de::try_take_isize);

/// Ensure the maximum value of each unsigned type encodes to a full length varint,
/// and matches the expected partial final byte.
#[test]
fn varint_max_encodings() {
    assert_eq!(
        encode(|f| ser::try_push_u16(f, u16::MAX)),
        [0xFF, 0xFF, 0x03]
    );
    assert_eq!(
        encode(|f| ser::try_push_u32(f, u32::MAX)),
        [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]
    );
    assert_eq!(
        encode(|f| ser::try_push_u64(f, u64::MAX)),
        [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
    );
    let mut u128_max = vec![0xFF; 18];
    u128_max.push(0x03);
    assert_eq!(encode(|f| ser::try_push_u128(f, u128::MAX)), u128_max);
}

/// Ensure varints that aren't too many bytes, but exceed the container's max
/// value are rejected.
#[test]
fn varint_last_byte_overflow() {
    assert_eq!(
        de::try_take_u16(&mut slice(&[0xFF, 0xFF, 0x04])),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        de::try_take_u32(&mut slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0x10])),
        Err(DecodeError::BadVarint)
    );
    let mut u64_bad = vec![0xFF; 9];
    u64_bad.push(0x02);
    assert_eq!(
        de::try_take_u64(&mut slice(&u64_bad)),
        Err(DecodeError::BadVarint)
    );
    let mut u128_bad = vec![0xFF; 18];
    u128_bad.push(0x04);
    assert_eq!(
        de::try_take_u128(&mut slice(&u128_bad)),
        Err(DecodeError::BadVarint)
    );
}

/// Ensure a varint longer than the type's maximum encoded length is rejected.
#[test]
fn varint_too_long() {
    assert_eq!(
        de::try_take_u16(&mut slice(&[0x80, 0x80, 0x80, 0x00])),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        de::try_take_u32(&mut slice(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x00])),
        Err(DecodeError::BadVarint)
    );
}

/// Ensure non-canonical varints that still fit within the maximum encoded
/// length are accepted.
#[test]
fn varint_non_canonical_accepted() {
    assert_eq!(de::try_take_u16(&mut slice(&[0x80, 0x00])), Ok(0));
    assert_eq!(de::try_take_u16(&mut slice(&[0x81, 0x80, 0x00])), Ok(1));
}

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

/// Ensure `f32` and `f64` encode as their little-endian IEEE 754 bit patterns,
/// including special values like `NaN` or `Infinity`.
#[test]
fn floats() {
    let v = f32::from_bits(0xc200_0600);
    let b = encode(|f| ser::try_push_f32(f, v));
    assert_eq!(b, [0x00, 0x06, 0x00, 0xc2]);
    assert_eq!(de::try_take_f32(&mut slice(&b)), Ok(v));

    let v = -32.005859375f64;
    let b = encode(|f| ser::try_push_f64(f, v));
    assert_eq!(b, [0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x40, 0xc0]);
    assert_eq!(de::try_take_f64(&mut slice(&b)), Ok(v));

    for v in [0.0f32, -0.0, f32::MIN, f32::MAX, f32::INFINITY, f32::NAN] {
        let b = encode(|f| ser::try_push_f32(f, v));
        let out = de::try_take_f32(&mut slice(&b)).unwrap();
        assert_eq!(out.to_bits(), v.to_bits());
    }
    for v in [0.0f64, -0.0, f64::MIN, f64::MAX, f64::INFINITY, f64::NAN] {
        let b = encode(|f| ser::try_push_f64(f, v));
        let out = de::try_take_f64(&mut slice(&b)).unwrap();
        assert_eq!(out.to_bits(), v.to_bits());
    }
    assert_eq!(
        de::try_take_f32(&mut slice(&[0x00, 0x00, 0x00])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
}

/// Ensure `char`s round-trip correctly, and invalid contents are rejected.
#[test]
fn chars() {
    for (v, len) in [('a', 1), ('ß', 2), ('€', 3), ('🦀', 4)] {
        let b = roundtrip!(ser::try_push_char, de::try_take_char, v, v);
        assert_eq!(b[0] as usize, len);
        assert_eq!(b.len(), len + 1);
    }
    assert_eq!(
        de::try_take_char(&mut slice(&[0x05, 0xF0, 0x9F, 0xA6, 0x80, 0x00])),
        Err(DecodeError::BadChar)
    );
    assert_eq!(
        de::try_take_char(&mut slice(&[0x00])),
        Err(DecodeError::BadChar)
    );
    assert_eq!(
        de::try_take_char(&mut slice(&[0x02, b'a', b'b'])),
        Err(DecodeError::BadChar)
    );
    assert_eq!(
        de::try_take_char(&mut slice(&[0x04, b'a', 0xC3, 0x9F, b'b'])),
        Err(DecodeError::BadChar)
    );
    assert_eq!(
        de::try_take_char(&mut slice(&[0x01, 0xFF])),
        Err(DecodeError::BadChar)
    );
}

/// Ensure variable-length and borrowable items (`[u8]` and `str`) round trip
/// correctly.
#[test]
fn strings_and_bytes() {
    let b = roundtrip!(ser::try_push_str, de::try_take_str, "hElLo", "hElLo");
    assert_eq!(b, [0x05, b'h', b'E', b'l', b'L', b'o']);

    let data: &[u8] = &[0x01, 0x10];
    let b = roundtrip!(ser::try_push_bytes, de::try_take_bytes, data, data);
    assert_eq!(b, [0x02, 0x01, 0x10]);

    let long = "x".repeat(200);
    let b = roundtrip!(
        ser::try_push_str,
        de::try_take_str,
        long.as_str(),
        long.as_str()
    );
    assert_eq!(&b[..2], &[0xC8, 0x01]);

    assert_eq!(
        de::try_take_str(&mut slice(&[0x02, 0xC3, 0x28])),
        Err(DecodeError::BadUtf8)
    );
    assert_eq!(
        de::try_take_bytes(&mut slice(&[0x03, 0x01])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
}

/// Ensure `Option<T>` round-trips correctly, and invalid discriminants are
/// rejected.
#[test]
fn options_discriminants_lengths() {
    assert_eq!(encode(ser::try_push_option_none), [0x00]);
    assert_eq!(encode(ser::try_push_option_some), [0x01]);
    assert_eq!(de::try_take_option_discrim(&mut slice(&[0x00])), Ok(false));
    assert_eq!(de::try_take_option_discrim(&mut slice(&[0x01])), Ok(true));
    for i in 2..=u8::MAX {
        assert_eq!(
            de::try_take_option_discrim(&mut slice(&[i])),
            Err(DecodeError::BadOption)
        );
    }

    let b = roundtrip!(
        ser::try_push_discriminant,
        de::try_take_discriminant,
        300u32,
        300
    );
    assert_eq!(b, [0xAC, 0x02]);
    let b = roundtrip!(ser::try_push_length, de::try_take_length, 300usize, 300);
    assert_eq!(b, [0xAC, 0x02]);
}

/// Ensure zig-zag encoded values round-trip as expected.
#[test]
fn zig_zag() {
    use postcard_core::varint::*;
    for v in [i16::MIN, -1, 0, 1, i16::MAX] {
        assert_eq!(de_zig_zag_i16(zig_zag_i16(v)), v);
    }
    for v in [i32::MIN, -1, 0, 1, i32::MAX] {
        assert_eq!(de_zig_zag_i32(zig_zag_i32(v)), v);
    }
    for v in [i64::MIN, -1, 0, 1, i64::MAX] {
        assert_eq!(de_zig_zag_i64(zig_zag_i64(v)), v);
    }
    for v in [i128::MIN, -1, 0, 1, i128::MAX] {
        assert_eq!(de_zig_zag_i128(zig_zag_i128(v)), v);
    }
    assert_eq!(zig_zag_i32(i32::MIN), u32::MAX);
    assert_eq!(zig_zag_i32(i32::MAX), u32::MAX - 1);
}

/// Ensure `varint_max` gives the maximum encoded length of each unsigned varint
/// type specified by the spec.
#[test]
fn varint_max_sizes() {
    use postcard_core::varint::varint_max;
    assert_eq!(varint_max::<u16>(), 3);
    assert_eq!(varint_max::<u32>(), 5);
    assert_eq!(varint_max::<u64>(), 10);
    assert_eq!(varint_max::<u128>(), 19);
}

/// Smoke test the `Slice` deser flavor.
#[test]
fn de_slice() {
    use de::Flavor;
    let data = [1u8, 2, 3, 4, 5];
    let mut fl = slice(&data);
    assert_eq!(fl.size_hint(), Some(5));
    assert_eq!(fl.pop(), Ok(1));
    assert_eq!(fl.try_take_n(2), Ok(&[2u8, 3][..]));
    assert_eq!(fl.size_hint(), Some(2));
    assert_eq!(fl.try_take_n(3), Err(de::UnexpectedEnd));
    assert_eq!(fl.try_take_n(1), Ok(&[4u8][..]));
    assert_eq!(fl.finalize(), Ok(&[5u8][..]));

    let mut fl = slice(&[]);
    assert_eq!(fl.pop(), Err(de::UnexpectedEnd));
    assert_eq!(fl.try_take_n(0), Ok(&[] as &[u8]));
}

/// Smoke test the `Slice` ser flavor.
#[test]
fn ser_slice() {
    use ser::Flavor;
    let mut buf = [0u8; 4];
    let mut fl = ser::flavors::Slice::new(&mut buf);
    assert_eq!(fl.try_push(1), Ok(()));
    assert_eq!(fl.try_extend(&[2, 3]), Ok(()));
    assert_eq!(fl.try_extend(&[4, 5]), Err(ser::BufferFull));
    assert_eq!(fl.try_push(4), Ok(()));
    assert_eq!(fl.try_push(5), Err(ser::BufferFull));
    assert_eq!(fl.try_extend(&[]), Ok(()));
    assert_eq!(fl.finalize().unwrap(), &[1, 2, 3, 4]);

    let mut buf = [0u8; 4];
    let mut fl = ser::flavors::Slice::new(&mut buf);
    fl.try_push(9).unwrap();
    assert_eq!(fl.finalize().unwrap(), &[9]);

    let mut fl = ser::flavors::Slice::new(&mut []);
    assert_eq!(fl.try_push(1), Err(ser::BufferFull));
    assert_eq!(fl.finalize().unwrap(), &[] as &[u8]);
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
        (
            range(0x01, 0xFE),
            concat(&[&[0xFF], &range(0x01, 0xFE), &[0x00]]),
        ),
        (
            concat(&[&[0x00], &range(0x01, 0xFE)]),
            concat(&[&[0x01, 0xFF], &range(0x01, 0xFE), &[0x00]]),
        ),
        (
            range(0x01, 0xFF),
            concat(&[&[0xFF], &range(0x01, 0xFE), &[0x02, 0xFF, 0x00]]),
        ),
        (
            concat(&[&range(0x02, 0xFF), &[0x00]]),
            concat(&[&[0xFF], &range(0x02, 0xFF), &[0x01, 0x01, 0x00]]),
        ),
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
    assert_eq!(fl.written_mut(), &[] as &[u8]);
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
    assert_eq!(inner.finalize(), Ok(&[] as &[u8]));
    assert_eq!(s, "hi");
}

/// Ensure various bad encodings are reported correctly.
#[test]
fn decode_error_kinds() {
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
        de::try_take_str(&mut slice(&[0x02, b'h'])),
        Err(DecodeError::Pop(de::UnexpectedEnd))
    );
    assert_eq!(
        de::try_take_str(&mut slice(&[0x01, 0xFF])),
        Err(DecodeError::BadUtf8)
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
