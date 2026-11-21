use postcard2::{DeserializerError, from_bytes, take_from_bytes};
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
struct Pair {
    a: u8,
    b: u8,
}

#[derive(Debug, PartialEq, Deserialize)]
enum Variants {
    Tuple(u8, u8),
    Struct { a: u8, b: u8 },
}

// postcard[verify schema.aggregate.decode.count]
#[test]
fn aggregate_count() {
    // Each aggregate takes two values, leaving the third byte unread
    let input = [0x01, 0x02, 0x03];
    assert_eq!(
        take_from_bytes::<(u8, u8)>(&input).unwrap(),
        ((1, 2), [3].as_slice())
    );
    assert_eq!(
        take_from_bytes::<Pair>(&input).unwrap(),
        (Pair { a: 1, b: 2 }, [3].as_slice())
    );
    assert_eq!(
        take_from_bytes::<Variants>(&[0x00, 0x01, 0x02, 0x03]).unwrap(),
        (Variants::Tuple(1, 2), [3].as_slice())
    );
    assert_eq!(
        take_from_bytes::<Variants>(&[0x01, 0x01, 0x02, 0x03]).unwrap(),
        (Variants::Struct { a: 1, b: 2 }, [3].as_slice())
    );

    // Too few bytes for the aggregate
    assert!(from_bytes::<(u8, u8, u8)>(&[0x01, 0x02]).is_err());
    assert!(from_bytes::<Pair>(&[0x01]).is_err());
}

/// Verify that invalid discriminants are rejected
// postcard[verify schema.tagged-union.decode.unknown-discriminant]
#[test]
fn unknown_discriminant() {
    // `Variants` has discriminants 0 and 1. Try 2, and 200 (a 2-byte varint).
    for input in [[0x02, 0x01, 0x02].as_slice(), &[0xC8, 0x01, 0x01, 0x02]] {
        assert!(matches!(
            from_bytes::<Variants>(input),
            Err(DeserializerError::Custom(_))
        ));
    }
}

/// Verify that decoding is contextual to the output type
// postcard[verify schema.decode.types]
#[test]
fn decode_types() {
    let input = [0x01, 0x02];
    assert_eq!(from_bytes::<(u8, u8)>(&input).unwrap(), (1, 2));
    assert_eq!(from_bytes::<(bool, u8)>(&input).unwrap(), (true, 2)); // 0x01 as `true`
    assert_eq!(from_bytes::<Vec<u8>>(&input).unwrap(), vec![2]); // 0x01 as a length
    assert_eq!(from_bytes::<&str>(&[0x01, b'a']).unwrap(), "a");
    assert_eq!(take_from_bytes::<u16>(&input).unwrap(), (1, [2].as_slice())); // one varint

    // 0x02 is a valid `u8`, but not a valid `bool`
    assert_eq!(from_bytes::<u8>(&[0x02]).unwrap(), 2);
    assert!(from_bytes::<bool>(&[0x02]).is_err());
}
