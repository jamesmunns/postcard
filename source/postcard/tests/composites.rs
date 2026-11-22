use postcard::{DecodeError, DeserializerError, from_bytes, take_from_bytes};
use serde::{Deserialize, Serialize, Serializer};

mod common;
use common::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Three {
    a: u8,
    b: u16,
    c: bool,
}

// postcard[verify comp.aggregate.encoding]
#[test]
fn aggregate_encoding() {
    let v = Three {
        a: 1,
        b: 0x80,
        c: true,
    };
    roundtrip(v, &[0x01, 0x80, 0x01, 0x01]); // a, b (2 bytes), c
    roundtrip((0x80u16, 2u8, false), &[0x80, 0x01, 0x02, 0x00]);
}

/// A struct encodes the same as a tuple of its fields.
// postcard[verify comp.aggregate.encoding]
#[test]
fn aggregate_no_length() {
    let s = encode(&Three {
        a: 7,
        b: 8,
        c: false,
    });
    let t = encode(&(7u8, 8u16, false));
    assert_eq!(s, t);
    assert_eq!(s, [0x07, 0x08, 0x00]);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct EmptyStruct {}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct EmptyTupleStruct();

// postcard[verify comp.aggregate.encoding]
#[test]
fn aggregate_empty() {
    roundtrip(EmptyStruct {}, &[]);
    roundtrip(EmptyTupleStruct(), &[]);
    assert_eq!(encode(&EmptyStruct {}), encode(&()));
}

// postcard[verify comp.sequence.encoding]
#[test]
fn sequence_encoding() {
    roundtrip(Vec::<u16>::new(), &[0x00]);
    roundtrip(vec![0x80u16, 0x01], &[0x02, 0x80, 0x01, 0x01]); // length, then each u16
}

// postcard[verify comp.sequence.decode.count]
#[test]
fn sequence_decode_count() {
    // The byte after the two elements is left unread
    let (v, rest) = take_from_bytes::<Vec<u8>>(&[0x02, 0x07, 0x08, 0x09]).unwrap();
    assert_eq!(v, [0x07, 0x08]);
    assert_eq!(rest, [0x09]);

    assert!(from_bytes::<Vec<u8>>(&[0x03, 0x07, 0x08]).is_err()); // length 3, two elements
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Small {
    A(u8),
    B(u16),
}

/// Pretend to be an enum with 200 variants (instead of making one and using
/// `derive(...)` on it).
struct Variant200(u8);

impl Serialize for Variant200 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // yup we're definitely the 200th enum variant, nothing to see here.
        s.serialize_newtype_variant("Variant200", 200, "V", &self.0)
    }
}

// postcard[verify comp.tagged-union.encoding]
#[test]
fn tagged_union_encoding() {
    roundtrip(Small::A(5), &[0x00, 0x05]); // variant 0
    roundtrip(Small::B(0x80), &[0x01, 0x80, 0x01]); // variant 1
    assert_eq!(encode(&Variant200(5)), [0xC8, 0x01, 0x05]); // 2-byte discriminant
}

// postcard[verify comp.option.encoding]
#[test]
fn option_encoding() {
    roundtrip(None::<u16>, &[0x00]);
    roundtrip(Some(0x80u16), &[0x01, 0x80, 0x01]);
}

// postcard[verify comp.option.encoding]
#[test]
fn option_rejects_invalid() {
    for i in 2..=u8::MAX {
        assert_eq!(
            from_bytes::<Option<u8>>(&[i, 0x00]),
            Err(DeserializerError::Decode(DecodeError::BadOption))
        );
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Shape {
    Empty,
    Point(u16, u16),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Nested {
    list: Vec<u16>,
    maybe: Option<u16>,
    shape: Shape,
    pair: (u8, bool),
}

/// Composites nested in an aggregate encode as expected and in order.
// postcard[verify comp.aggregate.encoding]
// postcard[verify comp.sequence.encoding]
// postcard[verify comp.tagged-union.encoding]
// postcard[verify comp.option.encoding]
#[test]
fn composite_encoding() {
    let val = Nested {
        list: vec![1, 0x80],
        maybe: Some(3),
        shape: Shape::Point(4, 5),
        pair: (6, true),
    };
    let parts = [
        encode(&val.list),
        encode(&val.maybe),
        encode(&val.shape),
        encode(&val.pair),
    ]
    .concat();
    assert_eq!(
        parts,
        [
            0x02, 0x01, 0x80, 0x01, // list: length, 1, 0x80
            0x01, 0x03, // maybe: Some, 3
            0x01, 0x04, 0x05, // shape: Point, 4, 5
            0x06, 0x01, // pair: 6, true
        ]
    );
    roundtrip(val, &parts);
}
