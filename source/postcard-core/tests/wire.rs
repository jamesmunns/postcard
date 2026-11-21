use postcard_core::de::DecodeError;
use postcard_core::{de, ser};

#[macro_use]
mod common;
use common::*;

/// Ensure that encoding a `()` pushes zero bytes.
// postcard[verify atom.unit.encoding]
#[test]
fn unit_encoding() {
    assert_eq!(encode(|f| ser::try_push_unit(f, ())), [] as [u8; 0]);
    let b = encode(|f| {
        ser::try_push_u8(f, 0xAA)?;
        ser::try_push_unit(f, ())?;
        ser::try_push_u8(f, 0xBB)
    });
    assert_eq!(b, [0xAA, 0xBB]);
}

/// Ensure that decoding a `()` consumes zero bytes.
// postcard[verify atom.unit.decode.empty]
#[test]
fn unit_decode() {
    let mut fl = slice(&[]);
    assert_eq!(de::try_take_unit(&mut fl), Ok(()));
    assert_eq!(de::Flavor::finalize(fl).unwrap(), [].as_slice());

    // Any following byte is left unread
    for i in 0..=u8::MAX {
        let bytes = [i];
        let mut fl = slice(&bytes);
        assert_eq!(de::try_take_unit(&mut fl), Ok(()));
        assert_eq!(de::Flavor::finalize(fl).unwrap(), &[i]);
    }
}

// postcard[verify atom.bool.encoding]
#[test]
fn bool_encoding() {
    let b = roundtrip!(ser::try_push_bool, de::try_take_bool, false, false);
    assert_eq!(b, [0x00]);
    let b = roundtrip!(ser::try_push_bool, de::try_take_bool, true, true);
    assert_eq!(b, [0x01]);
}

// postcard[verify atom.bool.decode.invalid]
#[test]
fn bool_rejects_invalid() {
    for i in 2..=u8::MAX {
        assert_eq!(
            de::try_take_bool(&mut slice(&[i])),
            Err(DecodeError::BadBool)
        );
    }
}

// postcard[verify atom.u8.encoding]
#[test]
fn u8_encoding() {
    for v in 0..=u8::MAX {
        let b = roundtrip!(ser::try_push_u8, de::try_take_u8, v, v);
        assert_eq!(b, [v]);
    }
}

// postcard[verify atom.u8.decode.any]
#[test]
fn u8_decode_any() {
    for v in 0..=u8::MAX {
        let bytes = [v, 0xAA];
        let mut fl = slice(&bytes);
        assert_eq!(de::try_take_u8(&mut fl), Ok(v));
        assert_eq!(de::Flavor::finalize(fl).unwrap(), &[0xAA]);
    }
}

// postcard[verify atom.i8.decode.any]
#[test]
fn i8_decode_any() {
    for v in 0..=u8::MAX {
        let bytes = [v, 0xAA];
        let mut fl = slice(&bytes);
        assert_eq!(de::try_take_i8(&mut fl), Ok(v as i8));
        assert_eq!(de::Flavor::finalize(fl).unwrap(), &[0xAA]);
    }
}

// postcard[verify atom.i8.encoding]
#[test]
fn i8_twos_complement() {
    for (v, byte) in [
        (0i8, 0x00),
        (1, 0x01),
        (127, 0x7F),
        (-1, 0xFF),
        (-128, 0x80),
    ] {
        let b = roundtrip!(ser::try_push_i8, de::try_take_i8, v, v);
        assert_eq!(b, [byte]);
    }
}

/// Test the examples from the spec's "Unsigned Integer Encoding" table.
// postcard[verify atom.varint.encoding.continuation]
// postcard[verify atom.varint.encoding.little-endian]
#[test]
fn unsigned_spec_examples() {
    let u16s: &[(u16, &[u8])] = &[
        (0, &[0x00]),
        (127, &[0x7F]),               // largest 1-byte
        (128, &[0x80, 0x01]),         // smallest 2-byte
        (16383, &[0xFF, 0x7F]),       // largest 2-byte
        (16384, &[0x80, 0x80, 0x01]), // smallest 3-byte
        (16385, &[0x81, 0x80, 0x01]), // low bit in first byte
        (65535, &[0xFF, 0xFF, 0x03]), // max, partial last byte
    ];
    for &(v, expected) in u16s {
        let b = roundtrip!(ser::try_push_u16, de::try_take_u16, v, v);
        assert_eq!(b, expected, "encoding of {v}");
    }
}

/// Test the examples from the spec's "Signed Integer Encoding" table.
// postcard[verify atom.varint.encoding.zigzag]
#[test]
fn signed_spec_examples() {
    let i16s: &[(i16, &[u8])] = &[
        (0, &[0x00]),
        (-1, &[0x01]),                 // negatives are odd
        (1, &[0x02]),                  // positives are even
        (63, &[0x7E]),                 // largest positive 1-byte
        (-64, &[0x7F]),                // most negative 1-byte
        (64, &[0x80, 0x01]),           // smallest positive 2-byte
        (-65, &[0x81, 0x01]),          // least negative 2-byte
        (32767, &[0xFE, 0xFF, 0x03]),  // i16::MAX
        (-32768, &[0xFF, 0xFF, 0x03]), // i16::MIN
    ];
    for &(v, expected) in i16s {
        let b = roundtrip!(ser::try_push_i16, de::try_take_i16, v, v);
        assert_eq!(b, expected, "encoding of {v}");
    }
}

/// Ensure decoding stops at the first byte without the continuation flag.
// postcard[verify atom.varint.encoding.continuation]
#[test]
fn varint_continuation_flag() {
    for v in [0u32, 1, 127, 128, 16383, 16384, u32::MAX] {
        let b = encode(|f| ser::try_push_u32(f, v));
        let (last, rest) = b.split_last().unwrap();
        assert_eq!(last & 0x80, 0, "last byte of {v}");
        assert!(rest.iter().all(|b| b & 0x80 != 0), "other bytes of {v}");
    }

    // The trailing 0xAA is left unread
    let data = [0x7F, 0xAA];
    let mut fl = slice(&data);
    assert_eq!(de::try_take_u32(&mut fl), Ok(127));
    assert_eq!(de::Flavor::finalize(fl), Ok([0xAA].as_slice()));

    let data = [0x80, 0x01, 0xAA];
    let mut fl = slice(&data);
    assert_eq!(de::try_take_u32(&mut fl), Ok(128));
    assert_eq!(de::Flavor::finalize(fl), Ok([0xAA].as_slice()));
}

// postcard[verify atom.varint.encoding.little-endian]
#[test]
fn varint_little_endian_groups() {
    let low = 0x2A; // bits 0-6
    let mid = 0x55; // bits 7-13
    let high = 0x13; // bits 14-20
    let v: u32 = low | (mid << 7) | (high << 14);
    let b = encode(|f| ser::try_push_u32(f, v));
    assert_eq!(b, [0x80 | low as u8, 0x80 | mid as u8, high as u8]);
    assert_eq!(de::try_take_u32(&mut slice(&b)), Ok(v));
}

macro_rules! varint_bounds {
    ($name:ident, $ty:ty, $push:path, $take:path) => {
        mod $name {
            use super::*;

            const VALUES: [$ty; 10] = [
                <$ty>::MIN,
                <$ty>::MIN + 1,
                0,
                1,
                63,  // largest 1-byte after zigzag
                64,  // smallest 2-byte after zigzag
                127, // largest 1-byte unsigned
                128, // smallest 2-byte unsigned
                <$ty>::MAX - 1,
                <$ty>::MAX,
            ];

            // postcard[verify atom.varint.types]
            #[test]
            fn roundtrip_as_varint() {
                // A varint, not a fixed-width integer
                assert_eq!(encode(|f| $push(f, 1)).len(), 1);
                for v in VALUES {
                    roundtrip!($push, $take, v, v);
                }
            }

            // postcard[verify atom.varint.max-len]
            #[test]
            fn within_max_len() {
                let max_bytes = postcard_core::varint::varint_max::<$ty>();
                for v in VALUES {
                    assert!(encode(|f| $push(f, v)).len() <= max_bytes);
                }
            }

            /// Ensure a canonical varint has no trailing `0x00` byte.
            // postcard[verify atom.varint.encode.canonical]
            #[test]
            fn canonical() {
                for v in VALUES {
                    let bytes = encode(|f| $push(f, v));
                    assert!(bytes.len() == 1 || bytes[bytes.len() - 1] != 0);
                }
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
// postcard[verify atom.varint.max-len]
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
// postcard[verify atom.varint.decode.max-value]
#[test]
fn varint_last_byte_overflow() {
    // Test the spec's canonicalization table
    assert_eq!(
        // u16's last byte can be at most 0x03
        de::try_take_u16(&mut slice(&[0xFF, 0xFF, 0x07])),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        de::try_take_u16(&mut slice(&[0xFF, 0xFF, 0x04])),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        // u32's last byte can be at most 0x0F
        de::try_take_u32(&mut slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0x10])),
        Err(DecodeError::BadVarint)
    );
    // u64's last byte can be at most 0x01
    let mut u64_bad = vec![0xFF; 9];
    u64_bad.push(0x02);
    assert_eq!(
        de::try_take_u64(&mut slice(&u64_bad)),
        Err(DecodeError::BadVarint)
    );
    // u128's last byte can be at most 0x03
    let mut u128_bad = vec![0xFF; 18];
    u128_bad.push(0x04);
    assert_eq!(
        de::try_take_u128(&mut slice(&u128_bad)),
        Err(DecodeError::BadVarint)
    );
}

/// Ensure a varint longer than the type's maximum encoded length is rejected.
// postcard[verify atom.varint.decode.max-len]
#[test]
fn varint_too_long() {
    // Test the spec's canonicalization table
    assert_eq!(
        // 4 bytes, u16 allows 3
        de::try_take_u16(&mut slice(&[0x80, 0x80, 0x80, 0x00])),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        // 3rd byte has the continuation flag
        de::try_take_u16(&mut slice(&[0xFF, 0xFF, 0x83, 0x00])),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        // 6 bytes, u32 allows 5
        de::try_take_u32(&mut slice(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x00])),
        Err(DecodeError::BadVarint)
    );
}

/// Ensure non-canonical varints that still fit within the maximum encoded
/// length are accepted.
// postcard[verify atom.varint.decode.non-canonical]
#[test]
fn varint_non_canonical_accepted() {
    // Test the spec's canonicalization table
    assert_eq!(de::try_take_u16(&mut slice(&[0x80, 0x00])), Ok(0)); // 0 in 2 bytes
    assert_eq!(de::try_take_u16(&mut slice(&[0x80, 0x80, 0x00])), Ok(0)); // 0 in 3 bytes
    assert_eq!(de::try_take_u16(&mut slice(&[0x81, 0x80, 0x00])), Ok(1)); // 1 in 3 bytes
}

// postcard[verify atom.f32.encoding]
#[test]
fn f32_encoding() {
    // The spec's example value, -32.005859375
    let v = f32::from_bits(0xc200_0600);
    let b = encode(|f| ser::try_push_f32(f, v));
    assert_eq!(b, [0x00, 0x06, 0x00, 0xc2]);
    assert_eq!(de::try_take_f32(&mut slice(&b)), Ok(v));

    // Special values, compared bit for bit as NaN != NaN
    for v in [0.0f32, -0.0, f32::MIN, f32::MAX, f32::INFINITY, f32::NAN] {
        let b = encode(|f| ser::try_push_f32(f, v));
        assert_eq!(b, v.to_bits().to_le_bytes());
        let out = de::try_take_f32(&mut slice(&b)).unwrap();
        assert_eq!(out.to_bits(), v.to_bits());
    }
}

// postcard[verify atom.f64.encoding]
#[test]
fn f64_encoding() {
    // The spec's example value
    let v = -32.005859375f64;
    let b = encode(|f| ser::try_push_f64(f, v));
    assert_eq!(b, [0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x40, 0xc0]);
    assert_eq!(de::try_take_f64(&mut slice(&b)), Ok(v));

    // Special values, compared bit for bit as NaN != NaN
    for v in [0.0f64, -0.0, f64::MIN, f64::MAX, f64::INFINITY, f64::NAN] {
        let b = encode(|f| ser::try_push_f64(f, v));
        assert_eq!(b, v.to_bits().to_le_bytes());
        let out = de::try_take_f64(&mut slice(&b)).unwrap();
        assert_eq!(out.to_bits(), v.to_bits());
    }
}

// postcard[verify atom.char.encoding]
#[test]
fn char_encoding() {
    // Test a `char` of each UTF-8 length.
    for v in ['a', 'ß', '€', '🦀'] {
        let mut buf = [0u8; 4];
        let as_str = encode(|f| ser::try_push_str(f, v.encode_utf8(&mut buf)));
        let b = roundtrip!(ser::try_push_char, de::try_take_char, v, v);
        assert_eq!(b, as_str);
    }
}

// postcard[verify atom.char.decode.one-scalar]
#[test]
fn char_rejects_not_one_scalar() {
    // Too long scalar values are refused
    assert_eq!(
        de::try_take_char(&mut slice(&[0x05, 0xF0, 0x9F, 0xA6, 0x80, 0x00])),
        Err(DecodeError::BadChar)
    );
    // Empty `char`s are refused
    assert_eq!(
        de::try_take_char(&mut slice(&[0x00])),
        Err(DecodeError::BadChar)
    );
    // Multiple scalar values are refused
    assert_eq!(
        de::try_take_char(&mut slice(&[0x02, b'a', b'b'])),
        Err(DecodeError::BadChar)
    );
    assert_eq!(
        de::try_take_char(&mut slice(&[0x04, b'a', 0xC3, 0x9F, b'b'])),
        Err(DecodeError::BadChar)
    );
    // Invalid UTF-8 refused
    assert_eq!(
        de::try_take_char(&mut slice(&[0x01, 0xFF])),
        Err(DecodeError::BadChar)
    );
}

// postcard[verify atom.bytes.encoding]
#[test]
fn bytes_encoding() {
    let data: &[u8] = &[0x01, 0x10];
    let b = roundtrip!(ser::try_push_bytes, de::try_take_bytes, data, data);
    assert_eq!(b, [0x02, 0x01, 0x10]);

    // 200 bytes needs a 2-byte length
    let long = [0xAAu8; 200];
    let b = roundtrip!(
        ser::try_push_bytes,
        de::try_take_bytes,
        long.as_slice(),
        long.as_slice()
    );
    assert_eq!(&b[..2], &[0xC8, 0x01]);
    assert_eq!(&b[2..], long.as_slice());
}

// postcard[verify atom.string.encoding]
#[test]
fn string_encoding() {
    let b = roundtrip!(ser::try_push_str, de::try_take_str, "hElLo", "hElLo");
    assert_eq!(b, [0x05, b'h', b'E', b'l', b'L', b'o']);

    // 200 bytes needs a 2-byte length
    let long = "x".repeat(200);
    let b = roundtrip!(
        ser::try_push_str,
        de::try_take_str,
        long.as_str(),
        long.as_str()
    );
    assert_eq!(&b[..2], &[0xC8, 0x01]);
    assert_eq!(&b[2..], long.as_bytes());
}

// postcard[verify atom.string.decode.utf8]
#[test]
fn string_rejects_invalid_utf8() {
    assert_eq!(
        // 0xC3 starts a 2-byte sequence, 0x28 can't continue it
        de::try_take_str(&mut slice(&[0x02, 0xC3, 0x28])),
        Err(DecodeError::BadUtf8)
    );
    assert_eq!(
        // 0xFF never appears in UTF-8
        de::try_take_str(&mut slice(&[0x01, 0xFF])),
        Err(DecodeError::BadUtf8)
    );
}

// postcard[verify atom.discriminant.encoding]
#[test]
fn discriminant_encoding() {
    for v in [0u32, 1, 127, 128, 300, u32::MAX] {
        let b = roundtrip!(ser::try_push_discriminant, de::try_take_discriminant, v, v);
        assert_eq!(b, encode(|f| ser::try_push_u32(f, v)));
    }
}

// postcard[verify atom.length.encoding]
#[test]
fn length_prefix() {
    for v in [0usize, 1, 127, 128, 300, usize::MAX] {
        let b = roundtrip!(ser::try_push_length, de::try_take_length, v, v);
        assert_eq!(b, encode(|f| ser::try_push_usize(f, v)));
    }
}

/// Ensure zig-zag encoded values round-trip as expected.
// postcard[verify atom.varint.encoding.zigzag]
#[test]
fn zig_zag() {
    use postcard_core::varint::*;
    // Decoding undoes encoding
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

    // Check the known values for each bit-width integer signed/unsigned pair.
    for (v, z) in [(0i16, 0u16), (-1, 1), (1, 2), (-2, 3), (2, 4)] {
        assert_eq!(zig_zag_i16(v), z);
        assert_eq!(de_zig_zag_i16(z), v);
    }
    assert_eq!(zig_zag_i16(i16::MIN), u16::MAX);
    assert_eq!(zig_zag_i16(i16::MAX), u16::MAX - 1);
    for (v, z) in [(0i32, 0u32), (-1, 1), (1, 2), (-2, 3), (2, 4)] {
        assert_eq!(zig_zag_i32(v), z);
    }
    assert_eq!(zig_zag_i32(i32::MIN), u32::MAX);
    assert_eq!(zig_zag_i32(i32::MAX), u32::MAX - 1);
    for (v, z) in [(0i64, 0u64), (-1, 1), (1, 2), (-2, 3), (2, 4)] {
        assert_eq!(zig_zag_i64(v), z);
        assert_eq!(de_zig_zag_i64(z), v);
    }
    assert_eq!(zig_zag_i64(i64::MIN), u64::MAX);
    assert_eq!(zig_zag_i64(i64::MAX), u64::MAX - 1);
    for (v, z) in [(0i128, 0u128), (-1, 1), (1, 2), (-2, 3), (2, 4)] {
        assert_eq!(zig_zag_i128(v), z);
        assert_eq!(de_zig_zag_i128(z), v);
    }
    assert_eq!(zig_zag_i128(i128::MIN), u128::MAX);
    assert_eq!(zig_zag_i128(i128::MAX), u128::MAX - 1);
}

/// Create a vec with `n` bytes of value `byte`, followed by `last`.
fn run(byte: u8, n: usize, last: u8) -> Vec<u8> {
    let mut v = vec![byte; n];
    v.push(last);
    v
}

/// Boundary tests for min/max + 7-bit varint increments.
// postcard[verify atom.varint.types]
// postcard[verify atom.varint.encoding.zigzag]
// postcard[verify conformance.encode.valid]
#[test]
fn varint_boundaries() {
    for (v, b) in [
        (0u16, vec![0x00]),                 // smallest
        (127, vec![0x7F]),                  // largest 1-byte
        (128, vec![0x80, 0x01]),            // smallest 2-byte
        (16383, vec![0xFF, 0x7F]),          // largest 2-byte
        (16384, vec![0x80, 0x80, 0x01]),    // smallest 3-byte
        (u16::MAX, vec![0xFF, 0xFF, 0x03]), // max, partial last byte
    ] {
        assert_eq!(roundtrip!(ser::try_push_u16, de::try_take_u16, v, v), b);
    }
    for (v, b) in [
        ((1u32 << 21) - 1, vec![0xFF, 0xFF, 0x7F]), // largest 3-byte
        (1 << 21, vec![0x80, 0x80, 0x80, 0x01]),    // smallest 4-byte
        ((1 << 28) - 1, vec![0xFF, 0xFF, 0xFF, 0x7F]), // largest 4-byte
        (1 << 28, vec![0x80, 0x80, 0x80, 0x80, 0x01]), // smallest 5-byte
        (u32::MAX, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]), // max, partial last byte
    ] {
        assert_eq!(roundtrip!(ser::try_push_u32, de::try_take_u32, v, v), b);
    }
    for (v, b) in [
        ((1u64 << 56) - 1, run(0xFF, 7, 0x7F)), // largest 8-byte
        (1 << 56, run(0x80, 8, 0x01)),          // smallest 9-byte
        ((1 << 63) - 1, run(0xFF, 8, 0x7F)),    // largest 9-byte
        (1 << 63, run(0x80, 9, 0x01)),          // top bit only
        (u64::MAX, run(0xFF, 9, 0x01)),         // max, partial last byte
    ] {
        assert_eq!(roundtrip!(ser::try_push_u64, de::try_take_u64, v, v), b);
    }
    for (v, b) in [
        ((1u128 << 126) - 1, run(0xFF, 17, 0x7F)), // largest 18-byte
        (1 << 126, run(0x80, 18, 0x01)),           // smallest 19-byte
        (1 << 127, run(0x80, 18, 0x02)),           // top bit only
        (u128::MAX, run(0xFF, 18, 0x03)),          // max, partial last byte
    ] {
        assert_eq!(roundtrip!(ser::try_push_u128, de::try_take_u128, v, v), b);
    }

    // Signed types are zigzag encoded, so MIN and MAX use the full width
    for (v, b) in [
        (i16::MIN, vec![0xFF, 0xFF, 0x03]), // zigzags to u16::MAX
        (i16::MAX, vec![0xFE, 0xFF, 0x03]), // zigzags to u16::MAX - 1
    ] {
        assert_eq!(roundtrip!(ser::try_push_i16, de::try_take_i16, v, v), b);
    }
    for (v, b) in [
        (-1i32, vec![0x01]),     // zigzags to 1
        (-64, vec![0x7F]),       // zigzags to 127, largest 1-byte
        (-65, vec![0x81, 0x01]), // zigzags to 129
        (i32::MIN, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
        (i32::MAX, vec![0xFE, 0xFF, 0xFF, 0xFF, 0x0F]),
    ] {
        assert_eq!(roundtrip!(ser::try_push_i32, de::try_take_i32, v, v), b);
    }
    for (v, b) in [
        (i64::MIN, run(0xFF, 9, 0x01)),
        (i64::MAX, [vec![0xFE], run(0xFF, 8, 0x01)].concat()),
    ] {
        assert_eq!(roundtrip!(ser::try_push_i64, de::try_take_i64, v, v), b);
    }
    for (v, b) in [
        (i128::MIN, run(0xFF, 18, 0x03)),
        (i128::MAX, [vec![0xFE], run(0xFF, 17, 0x03)].concat()),
    ] {
        assert_eq!(roundtrip!(ser::try_push_i128, de::try_take_i128, v, v), b);
    }

    // usize and isize take the width of the platform's pointer
    #[cfg(target_pointer_width = "64")]
    {
        let b = roundtrip!(
            ser::try_push_usize,
            de::try_take_usize,
            usize::MAX,
            usize::MAX
        );
        assert_eq!(b, run(0xFF, 9, 0x01));
        let b = roundtrip!(
            ser::try_push_isize,
            de::try_take_isize,
            isize::MIN,
            isize::MIN
        );
        assert_eq!(b, run(0xFF, 9, 0x01));
    }
    #[cfg(target_pointer_width = "32")]
    {
        let b = roundtrip!(
            ser::try_push_usize,
            de::try_take_usize,
            usize::MAX,
            usize::MAX
        );
        assert_eq!(b, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        let b = roundtrip!(
            ser::try_push_isize,
            de::try_take_isize,
            isize::MIN,
            isize::MIN
        );
        assert_eq!(b, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }
}

/// Ensure `varint_max` gives the maximum encoded length of each unsigned varint
/// type specified by the spec.
// postcard[verify atom.varint.max-len]
#[test]
fn varint_max_sizes() {
    use postcard_core::varint::varint_max;
    assert_eq!(varint_max::<u16>(), 3);
    assert_eq!(varint_max::<u32>(), 5);
    assert_eq!(varint_max::<u64>(), 10);
    assert_eq!(varint_max::<u128>(), 19);
}

/// Test that "too big" `usize`s fail to decode
// postcard[verify atom.varint.decode.max-value]
#[test]
fn usize_overflow() {
    #[cfg(target_pointer_width = "16")]
    let big = encode(|f| ser::try_push_u32(f, u16::MAX as u32 + 1));
    #[cfg(target_pointer_width = "32")]
    let big = encode(|f| ser::try_push_u64(f, u32::MAX as u64 + 1));
    #[cfg(target_pointer_width = "64")]
    let big = encode(|f| ser::try_push_u128(f, u64::MAX as u128 + 1));

    assert_eq!(
        de::try_take_usize(&mut slice(&big)),
        Err(DecodeError::BadVarint)
    );
}

// postcard[verify atom.usize.encoding]
#[test]
fn usize_matches_pointer_width() {
    for v in [0usize, 1, 127, 128, usize::MAX] {
        let b = encode(|f| ser::try_push_usize(f, v));
        #[cfg(target_pointer_width = "32")]
        assert_eq!(b, encode(|f| ser::try_push_u32(f, v as u32)));
        #[cfg(target_pointer_width = "64")]
        assert_eq!(b, encode(|f| ser::try_push_u64(f, v as u64)));
    }
}

// postcard[verify atom.isize.encoding]
#[test]
fn isize_matches_pointer_width() {
    for v in [0isize, -1, 1, isize::MIN, isize::MAX] {
        let b = encode(|f| ser::try_push_isize(f, v));
        #[cfg(target_pointer_width = "32")]
        assert_eq!(b, encode(|f| ser::try_push_i32(f, v as i32)));
        #[cfg(target_pointer_width = "64")]
        assert_eq!(b, encode(|f| ser::try_push_i64(f, v as i64)));
    }
}

/// Test that "too big" `isize`s fail to decode
// postcard[verify atom.varint.decode.max-value]
#[test]
fn isize_overflow() {
    #[cfg(target_pointer_width = "16")]
    let (min, max) = (i16::MIN as i32, i16::MAX as i32);
    #[cfg(target_pointer_width = "32")]
    let (min, max) = (i32::MIN as i64, i32::MAX as i64);
    #[cfg(target_pointer_width = "64")]
    let (min, max) = (i64::MIN as i128, i64::MAX as i128);

    for v in [min - 1, max + 1] {
        #[cfg(target_pointer_width = "16")]
        let big = encode(|f| ser::try_push_i32(f, v));
        #[cfg(target_pointer_width = "32")]
        let big = encode(|f| ser::try_push_i64(f, v));
        #[cfg(target_pointer_width = "64")]
        let big = encode(|f| ser::try_push_i128(f, v));

        assert_eq!(
            de::try_take_isize(&mut slice(&big)),
            Err(DecodeError::BadVarint),
            "decoding {v}"
        );
    }
}

// postcard[verify conformance.decode.truncated]
#[test]
fn decode_truncated() {
    use postcard_core::de::UnexpectedEnd;
    const END: DecodeError<UnexpectedEnd> = DecodeError::Pop(UnexpectedEnd);

    // Fixed size: no bytes at all
    assert_eq!(de::try_take_bool(&mut slice(&[])), Err(END));
    assert_eq!(de::try_take_u8(&mut slice(&[])), Err(END));
    assert_eq!(de::try_take_i8(&mut slice(&[])), Err(END));

    // Varints: the last byte has the continuation flag set
    assert_eq!(de::try_take_u16(&mut slice(&[0x80])), Err(END));
    assert_eq!(de::try_take_u32(&mut slice(&[0xFF, 0xFF])), Err(END));
    assert_eq!(de::try_take_u64(&mut slice(&[0x80])), Err(END));
    assert_eq!(de::try_take_u128(&mut slice(&[0x80])), Err(END));
    assert_eq!(de::try_take_usize(&mut slice(&[0x80])), Err(END));
    assert_eq!(de::try_take_i32(&mut slice(&[0x81])), Err(END));
    assert_eq!(de::try_take_isize(&mut slice(&[0x81])), Err(END));

    // Floats: one byte short
    assert_eq!(de::try_take_f32(&mut slice(&[0x00; 3])), Err(END));
    assert_eq!(de::try_take_f64(&mut slice(&[0x00; 7])), Err(END));

    // Length-prefixed: fewer bytes than the length says
    assert_eq!(
        de::try_take_bytes(&mut slice(&[0x03, 0x01, 0x02])),
        Err(END)
    );
    assert_eq!(de::try_take_str(&mut slice(&[0x02, b'a'])), Err(END));
    assert_eq!(de::try_take_char(&mut slice(&[0x02, 0xC3])), Err(END));

    // Headers: a varint cut short
    assert_eq!(de::try_take_discriminant(&mut slice(&[0x80])), Err(END));
    assert_eq!(de::try_take_length(&mut slice(&[0x80])), Err(END));
}

// postcard[verify atom.f32.decode.any]
// postcard[verify atom.f32.decode.nan]
#[test]
fn f32_decode_any() {
    let special = [
        0x7F80_0001, // signalling NaN
        0x7FC0_0001, // quiet NaN with a payload
        0xFFFF_FFFF, // negative NaN, all payload bits set
        0x0000_0001, // smallest subnormal
        0x8000_0000, // -0.0
    ];
    // A prime step, to sample unrelated bit patterns across the whole range
    let sweep = (0..=u32::MAX).step_by(65_521);
    for bits in special.into_iter().chain(sweep) {
        let out = de::try_take_f32(&mut slice(&bits.to_le_bytes())).unwrap();
        assert_eq!(out.to_bits(), bits);
    }
}

// postcard[verify atom.f64.decode.any]
// postcard[verify atom.f64.decode.nan]
#[test]
fn f64_decode_any() {
    let special = [
        0x7FF0_0000_0000_0001, // signalling NaN
        0x7FF8_0000_0000_0001, // quiet NaN with a payload
        0xFFFF_FFFF_FFFF_FFFF, // negative NaN, all payload bits set
        0x0000_0000_0000_0001, // smallest subnormal
        0x8000_0000_0000_0000, // -0.0
    ];
    // A large odd step, to sample bit patterns across the whole range
    let sweep = (0..=u64::MAX).step_by(0x0000_1000_0000_0F01);
    for bits in special.into_iter().chain(sweep) {
        let out = de::try_take_f64(&mut slice(&bits.to_le_bytes())).unwrap();
        assert_eq!(out.to_bits(), bits);
    }
}

// postcard[verify conformance.decode.reject]
#[test]
fn decode_reject() {
    assert_eq!(
        de::try_take_bool(&mut slice(&[0x02])),
        Err(DecodeError::BadBool)
    );
    assert_eq!(
        // too long
        de::try_take_u16(&mut slice(&[0x80, 0x80, 0x80, 0x00])),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        // too large
        de::try_take_u16(&mut slice(&[0xFF, 0xFF, 0x07])),
        Err(DecodeError::BadVarint)
    );
    assert_eq!(
        de::try_take_str(&mut slice(&[0x01, 0xFF])),
        Err(DecodeError::BadUtf8)
    );
    assert_eq!(
        de::try_take_char(&mut slice(&[0x02, b'a', b'b'])),
        Err(DecodeError::BadChar)
    );
    assert!(matches!(
        // truncated
        de::try_take_u32(&mut slice(&[0x80])),
        Err(DecodeError::Pop(_))
    ));
}

/// The number of bytes in the canonical `varint` encoding of `v`
fn canonical_len(v: u128) -> usize {
    let bits = 128 - v.leading_zeros() as usize;
    bits.div_ceil(7).max(1)
}

// postcard[verify conformance.encode.valid]
// postcard[verify conformance.decode.accept]
#[test]
fn encode_valid_decode_accept() {
    // Fixed size: every value
    for v in [false, true] {
        roundtrip!(ser::try_push_bool, de::try_take_bool, v, v);
    }
    for v in 0..=u8::MAX {
        roundtrip!(ser::try_push_u8, de::try_take_u8, v, v);
    }
    for v in i8::MIN..=i8::MAX {
        roundtrip!(ser::try_push_i8, de::try_take_i8, v, v);
    }

    // Unsigned varints, checking each is the canonical length
    for v in (0..=u16::MAX).step_by(7).chain([u16::MAX]) {
        let b = roundtrip!(ser::try_push_u16, de::try_take_u16, v, v);
        assert_eq!(b.len(), canonical_len(v.into()));
    }
    for v in (0..=u32::MAX).step_by(65_521).chain([u32::MAX]) {
        let b = roundtrip!(ser::try_push_u32, de::try_take_u32, v, v);
        assert_eq!(b.len(), canonical_len(v.into()));
    }
    // Each power of two, one less, and each run of low ones
    for shift in 0..64 {
        for v in [1u64 << shift, (1u64 << shift) - 1, u64::MAX >> shift] {
            let b = roundtrip!(ser::try_push_u64, de::try_take_u64, v, v);
            assert_eq!(b.len(), canonical_len(v.into()));
        }
    }
    for shift in 0..128 {
        for v in [1u128 << shift, (1u128 << shift) - 1, u128::MAX >> shift] {
            let b = roundtrip!(ser::try_push_u128, de::try_take_u128, v, v);
            assert_eq!(b.len(), canonical_len(v));
        }
    }
    for v in [0usize, 1, 127, 128, usize::MAX] {
        roundtrip!(ser::try_push_usize, de::try_take_usize, v, v);
    }

    // Signed varints
    for v in (i16::MIN..=i16::MAX).step_by(7).chain([i16::MAX]) {
        roundtrip!(ser::try_push_i16, de::try_take_i16, v, v);
    }
    for v in (i32::MIN..=i32::MAX).step_by(65_521).chain([i32::MAX]) {
        roundtrip!(ser::try_push_i32, de::try_take_i32, v, v);
    }
    for v in [i64::MIN, -1, 0, 1, i64::MAX] {
        roundtrip!(ser::try_push_i64, de::try_take_i64, v, v);
    }
    for v in [i128::MIN, -1, 0, 1, i128::MAX] {
        roundtrip!(ser::try_push_i128, de::try_take_i128, v, v);
    }
    for v in [isize::MIN, -1, 0, 1, isize::MAX] {
        roundtrip!(ser::try_push_isize, de::try_take_isize, v, v);
    }

    // Floats, without NaN as NaN != NaN (see `f32_decode_any`)
    for v in [0.0f32, -0.0, 1.5, f32::MIN, f32::MAX, f32::INFINITY] {
        roundtrip!(ser::try_push_f32, de::try_take_f32, v, v);
    }
    for v in [0.0f64, -0.0, 1.5, f64::MIN, f64::MAX, f64::INFINITY] {
        roundtrip!(ser::try_push_f64, de::try_take_f64, v, v);
    }

    // Length-prefixed atoms
    for v in [b"".as_slice(), b"\x00", &[0xFF; 300]] {
        roundtrip!(ser::try_push_bytes, de::try_take_bytes, v, v);
    }
    for v in ["", "a", "héllo", "\u{10FFFF}"] {
        roundtrip!(ser::try_push_str, de::try_take_str, v, v);
    }
    // Every 97th code point, skipping surrogates
    for v in (0..=0x10_FFFFu32).step_by(97).filter_map(char::from_u32) {
        roundtrip!(ser::try_push_char, de::try_take_char, v, v);
    }
    // Headers
    for v in [0u32, 1, 127, 128, u32::MAX] {
        roundtrip!(ser::try_push_discriminant, de::try_take_discriminant, v, v);
    }
    for v in [0usize, 1, 127, 128, usize::MAX] {
        roundtrip!(ser::try_push_length, de::try_take_length, v, v);
    }
}
