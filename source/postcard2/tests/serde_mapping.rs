use std::collections::BTreeMap;

use postcard2::{
    DecodeError, Deserializer, DeserializerError, SerializerError, from_bytes, to_slice,
};
use serde::de::IgnoredAny;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize, Serializer};

mod common;
use common::*;

/// Ensure `bytes` fail to decode as `T`, because the value they encode is out of
/// `T`'s range.
fn assert_out_of_range<T: for<'de> Deserialize<'de> + core::fmt::Debug + PartialEq>(bytes: &[u8]) {
    assert_eq!(
        from_bytes::<T>(bytes),
        Err(DeserializerError::Decode(DecodeError::BadVarint)),
        "decoding {bytes:?}"
    );
}

/// Create a vec with `n` bytes of value `byte`, followed by `last`.
fn run(byte: u8, n: usize, last: u8) -> Vec<u8> {
    let mut v = vec![byte; n];
    v.push(last);
    v
}

// postcard[verify serde.type.bool]
#[test]
fn bool() {
    roundtrip(false, &[0x00]);
    roundtrip(true, &[0x01]);
    // Every other byte is invalid
    for i in 2..=u8::MAX {
        assert_eq!(
            from_bytes::<bool>(&[i]),
            Err(DeserializerError::Decode(DecodeError::BadBool))
        );
    }
}

// postcard[verify serde.type.i8]
#[test]
fn i8() {
    // Always encoded as a single byte (and not a varint)
    for (v, b) in [
        (i8::MIN, 0x80),
        (-1, 0xFF),
        (0, 0x00),
        (1, 0x01),
        (i8::MAX, 0x7F),
    ] {
        roundtrip(v, &[b]);
    }
}

// postcard[verify serde.type.i16]
#[test]
fn i16() {
    for (v, b) in [
        (i16::MIN, vec![0xFF, 0xFF, 0x03]),
        (-1, vec![0x01]),
        (0, vec![0x00]),
        (1, vec![0x02]),
        (i16::MAX, vec![0xFE, 0xFF, 0x03]),
    ] {
        roundtrip(v, &b);
    }
    // One past either end, encoded as a wider type
    assert_out_of_range::<i16>(&encode(&(i16::MIN as i32 - 1)));
    assert_out_of_range::<i16>(&encode(&(i16::MAX as i32 + 1)));
}

// postcard[verify serde.type.i32]
#[test]
fn i32() {
    for (v, b) in [
        (i32::MIN, run(0xFF, 4, 0x0F)),
        (-1, vec![0x01]),
        (0, vec![0x00]),
        (1, vec![0x02]),
        (i32::MAX, [vec![0xFE], run(0xFF, 3, 0x0F)].concat()),
    ] {
        roundtrip(v, &b);
    }
    // One past either end, encoded as a wider type
    assert_out_of_range::<i32>(&encode(&(i32::MIN as i64 - 1)));
    assert_out_of_range::<i32>(&encode(&(i32::MAX as i64 + 1)));
}

// postcard[verify serde.type.i64]
#[test]
fn i64() {
    for (v, b) in [
        (i64::MIN, run(0xFF, 9, 0x01)),
        (-1, vec![0x01]),
        (0, vec![0x00]),
        (1, vec![0x02]),
        (i64::MAX, [vec![0xFE], run(0xFF, 8, 0x01)].concat()),
    ] {
        roundtrip(v, &b);
    }
    // One past either end, encoded as a wider type
    assert_out_of_range::<i64>(&encode(&(i64::MIN as i128 - 1)));
    assert_out_of_range::<i64>(&encode(&(i64::MAX as i128 + 1)));
}

// postcard[verify serde.type.i128]
#[test]
fn i128() {
    for (v, b) in [
        (i128::MIN, run(0xFF, 18, 0x03)),
        (-1, vec![0x01]),
        (0, vec![0x00]),
        (1, vec![0x02]),
        (i128::MAX, [vec![0xFE], run(0xFF, 17, 0x03)].concat()),
    ] {
        roundtrip(v, &b);
    }
    // One past the end of the range, which doesn't fit in an `i128`
    assert_out_of_range::<i128>(&run(0xFF, 18, 0x04));
}

// postcard[verify serde.type.u8]
#[test]
fn u8() {
    for v in [0, 1, u8::MAX] {
        roundtrip(v, &[v]);
    }
}

// postcard[verify serde.type.u16]
#[test]
fn u16() {
    for (v, b) in [
        (0, vec![0x00]),
        (1, vec![0x01]),
        (u16::MAX, vec![0xFF, 0xFF, 0x03]),
    ] {
        roundtrip(v, &b);
    }
    // One past the end, encoded as a wider type
    assert_out_of_range::<u16>(&encode(&(u16::MAX as u32 + 1)));
}

// postcard[verify serde.type.u32]
#[test]
fn u32() {
    for (v, b) in [
        (0, vec![0x00]),
        (1, vec![0x01]),
        (u32::MAX, run(0xFF, 4, 0x0F)),
    ] {
        roundtrip(v, &b);
    }
    // One past the end, encoded as a wider type
    assert_out_of_range::<u32>(&encode(&(u32::MAX as u64 + 1)));
}

// postcard[verify serde.type.u64]
#[test]
fn u64() {
    for (v, b) in [
        (0, vec![0x00]),
        (1, vec![0x01]),
        (u64::MAX, run(0xFF, 9, 0x01)),
    ] {
        roundtrip(v, &b);
    }
    // One past the end, encoded as a wider type
    assert_out_of_range::<u64>(&encode(&(u64::MAX as u128 + 1)));
}

// postcard[verify serde.type.u128]
#[test]
fn u128() {
    for (v, b) in [
        (0, vec![0x00]),
        (1, vec![0x01]),
        (u128::MAX, run(0xFF, 18, 0x03)),
    ] {
        roundtrip(v, &b);
    }
    // One past the end of the range, which doesn't fit in a `u128`
    assert_out_of_range::<u128>(&run(0xFF, 18, 0x04));
}

// postcard[verify serde.type.f32]
#[test]
fn f32() {
    // The spec's example value, -32.005859375
    roundtrip(f32::from_bits(0xc200_0600), &[0x00, 0x06, 0x00, 0xc2]);
}

// postcard[verify serde.type.f64]
#[test]
fn f64() {
    roundtrip(
        -32.005859375f64,
        &[0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x40, 0xc0],
    );
}

// postcard[verify serde.type.char]
#[test]
fn char() {
    roundtrip('é', &[0x02, 0xC3, 0xA9]);
    // Multiple characters
    assert_eq!(
        from_bytes::<char>(&[0x02, b'a', b'b']),
        Err(DeserializerError::Decode(DecodeError::BadChar))
    );
}

// postcard[verify serde.type.string]
#[test]
fn string() {
    roundtrip(String::from("hi"), &[0x02, b'h', b'i']);
    // Borrowed from the input
    let s: &str = postcard2::from_bytes(&[0x02, b'h', b'i']).unwrap();
    assert_eq!(s, "hi");
    // 0xFF never appears in UTF-8
    let bad = [0x01, 0xFF];
    let err = Err(DeserializerError::Decode(DecodeError::BadUtf8));
    assert_eq!(from_bytes::<String>(&bad), err);
    assert_eq!(from_bytes::<&str>(&bad).map(str::to_owned), err);
}

/// Serializes as a serde `byte array`, rather than a `seq` of `u8`, unlike
/// derived impls.
struct Bytes<'a>(&'a [u8]);

impl Serialize for Bytes<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(self.0)
    }
}

// postcard[verify serde.type.byte-array]
#[test]
fn byte_array() {
    assert_eq!(encode(&Bytes(&[0xAA, 0xBB])), [0x02, 0xAA, 0xBB]);
    // Borrowed from the input
    let b: &[u8] = postcard2::from_bytes(&[0x02, 0xAA, 0xBB]).unwrap();
    assert_eq!(b, [0xAA, 0xBB]);
}

// postcard[verify serde.type.option]
#[test]
fn option() {
    roundtrip(None::<u8>, &[0x00]);
    roundtrip(Some(0x05u8), &[0x01, 0x05]);
}

// postcard[verify serde.type.unit]
#[test]
fn unit() {
    roundtrip((), &[]);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct UnitStruct;

// postcard[verify serde.type.unit-struct]
#[test]
fn unit_struct() {
    roundtrip(UnitStruct, &[]);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Variants {
    Unit,
    Newtype(u16),
    Tuple(u8, u16),
    Struct { a: u8, b: u16 },
}

// postcard[verify serde.type.unit-variant]
#[test]
fn unit_variant() {
    roundtrip(Variants::Unit, &[0x00]); // just the discriminant
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Newtype(u16);

// postcard[verify serde.type.newtype-struct]
#[test]
fn newtype_struct() {
    roundtrip(Newtype(0x80), &[0x80, 0x01]); // just the u16
}

// postcard[verify serde.type.newtype-variant]
#[test]
fn newtype_variant() {
    roundtrip(Variants::Newtype(0x80), &[0x01, 0x80, 0x01]); // discriminant, then the u16
}

// postcard[verify serde.type.seq]
#[test]
fn seq() {
    roundtrip(vec![1u8, 2, 3], &[0x03, 0x01, 0x02, 0x03]);
}

/// Verify fixed-size arrays and tuples are encoded as the `tuple` type
// postcard[verify serde.type.tuple]
#[test]
fn tuple() {
    roundtrip((1u8, 0x80u16), &[0x01, 0x80, 0x01]);
    roundtrip([1u8, 2, 3], &[0x01, 0x02, 0x03]); // no length, unlike a `seq`
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TupleStruct(u8, u16);

// postcard[verify serde.type.tuple-struct]
#[test]
fn tuple_struct() {
    roundtrip(TupleStruct(1, 0x80), &[0x01, 0x80, 0x01]);
}

// postcard[verify serde.type.tuple-variant]
#[test]
fn tuple_variant() {
    roundtrip(Variants::Tuple(1, 0x80), &[0x02, 0x01, 0x80, 0x01]); // discriminant, then the fields
}

// postcard[verify serde.type.map]
#[test]
fn map() {
    let m = BTreeMap::from([(1u8, 0x80u16), (2, 3)]);
    roundtrip(m, &[0x02, 0x01, 0x80, 0x01, 0x02, 0x03]); // length, then key, value, key, value
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Struct {
    a: u8,
    b: u16,
}

// postcard[verify serde.type.struct]
#[test]
fn r#struct() {
    roundtrip(Struct { a: 1, b: 0x80 }, &[0x01, 0x80, 0x01]);
}

// postcard[verify serde.type.struct-variant]
#[test]
fn struct_variant() {
    roundtrip(
        Variants::Struct { a: 1, b: 0x80 },
        &[0x03, 0x01, 0x80, 0x01], // discriminant, then the fields
    );
}

/// A type that serializes as a `seq` but with no length hint.
struct UnknownSeq;

impl Serialize for UnknownSeq {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_seq(None)?.end()
    }
}

/// A type that serializes as a `map` but with no length hint.
struct UnknownMap;

impl Serialize for UnknownMap {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_map(None)?.end()
    }
}

/// Verify a missing length hint is a deal breaker
// postcard[verify serde.encode.length-required]
#[test]
fn length_required() {
    let mut buf = [0u8; 8];
    assert!(matches!(
        to_slice(&UnknownSeq, &mut buf),
        Err(SerializerError::SeqLengthUnknown)
    ));
    assert!(matches!(
        to_slice(&UnknownMap, &mut buf),
        Err(SerializerError::SeqLengthUnknown)
    ));
}

// postcard[verify serde.decode.any.unsupported]
#[test]
fn any_unsupported() {
    let mut de = Deserializer::from_bytes(&[0x00]);
    assert_eq!(
        serde::Deserializer::deserialize_any(&mut de, IgnoredAny),
        Err(DeserializerError::UnsupportedDeserAny)
    );
}

// postcard[verify serde.decode.identifier.unsupported]
#[test]
fn identifier_unsupported() {
    let mut de = Deserializer::from_bytes(&[0x00]);
    assert_eq!(
        serde::Deserializer::deserialize_identifier(&mut de, IgnoredAny),
        Err(DeserializerError::UnsupportedDeserIdent)
    );
}

// postcard[verify serde.decode.ignored-any.unsupported]
#[test]
fn ignored_any_unsupported() {
    let mut de = Deserializer::from_bytes(&[0x00]);
    assert_eq!(
        IgnoredAny::deserialize(&mut de),
        Err(DeserializerError::UnsupportedDeserIgnoredAny)
    );
}

/// Verify that we use lexical order, and NOT the repr discriminant
// postcard[verify serde.variant.discriminant]
#[test]
fn variant_discriminant() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[repr(u8)]
    enum Explicit {
        A = 5,
        B = 200,
    }
    // The `#[repr]` values 5 and 200 are not used
    roundtrip(Explicit::A, &[0x00]);
    roundtrip(Explicit::B, &[0x01]);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Kinds {
        Unit,
        Newtype(u8),
        Tuple(u8, u8),
        Struct { a: u8 },
    }
    // Each kind of variant, numbered in declaration order
    roundtrip(Kinds::Unit, &[0x00]);
    roundtrip(Kinds::Newtype(7), &[0x01, 0x07]);
    roundtrip(Kinds::Tuple(7, 8), &[0x02, 0x07, 0x08]);
    roundtrip(Kinds::Struct { a: 7 }, &[0x03, 0x07]);
}

/// A type that smuggles out whether the serializer/deserializer was human readable or not
#[derive(Debug, PartialEq)]
struct HumanReadable(bool);

impl Serialize for HumanReadable {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let hr = s.is_human_readable();
        s.serialize_bool(hr)
    }
}

impl<'de> Deserialize<'de> for HumanReadable {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(HumanReadable(d.is_human_readable()))
    }
}

// postcard[verify serde.human-readable]
#[test]
fn human_readable() {
    assert_eq!(encode(&HumanReadable(true)), [0x00]); // the serializer said `false`
    assert_eq!(
        postcard2::from_bytes::<HumanReadable>(&[]).unwrap(),
        HumanReadable(false)
    );
}

/// A visitor that only accepts a sequence
struct SeqOnlyVisitor;

impl<'de> serde::de::Visitor<'de> for SeqOnlyVisitor {
    type Value = (u8, u8);

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("two fields, as a sequence")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<(u8, u8), A::Error> {
        let a = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let b = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        Ok((a, b))
    }
}

/// A `struct` decoded through a visitor that only accepts sequences.
#[derive(Debug, PartialEq)]
struct SeqStruct(u8, u8);

impl<'de> Deserialize<'de> for SeqStruct {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (a, b) = d.deserialize_struct("SeqStruct", &["a", "b"], SeqOnlyVisitor)?;
        Ok(SeqStruct(a, b))
    }
}

/// A `struct_variant` decoded through a visitor that only accepts sequences.
#[derive(Debug, PartialEq)]
struct SeqVariant(u8, u8);

impl<'de> Deserialize<'de> for SeqVariant {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct EnumVisitor;
        impl<'de> serde::de::Visitor<'de> for EnumVisitor {
            type Value = SeqVariant;

            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str("a struct variant")
            }

            fn visit_enum<A: serde::de::EnumAccess<'de>>(
                self,
                data: A,
            ) -> Result<SeqVariant, A::Error> {
                use serde::de::VariantAccess;
                let (_index, variant): (u32, _) = data.variant()?;
                let (a, b) = variant.struct_variant(&["a", "b"], SeqOnlyVisitor)?;
                Ok(SeqVariant(a, b))
            }
        }
        d.deserialize_enum("SeqVariant", &["S"], EnumVisitor)
    }
}

// postcard[verify serde.decode.struct.seq]
#[test]
fn struct_as_seq() {
    assert_eq!(
        postcard2::from_bytes::<SeqStruct>(&[0x01, 0x02]).unwrap(),
        SeqStruct(1, 2)
    );
    assert_eq!(
        // discriminant, then the fields
        postcard2::from_bytes::<SeqVariant>(&[0x00, 0x01, 0x02]).unwrap(),
        SeqVariant(1, 2)
    );
}

/// A value that formats one byte longer each time it is formatted.
struct Growing(core::cell::Cell<usize>);

impl core::fmt::Display for Growing {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let n = self.0.get();
        self.0.set(n + 1);
        f.write_str(&"x".repeat(n))
    }
}

impl Serialize for Growing {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

/// Checks that a value that formats differently on each pass is an
/// error, rather than producing an incorrect length header
// postcard[verify serde.type.string]
#[test]
fn collect_str() {
    struct Fixed;
    impl core::fmt::Display for Fixed {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            f.write_str("hi")
        }
    }
    impl Serialize for Fixed {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            s.collect_str(self)
        }
    }
    assert_eq!(encode(&Fixed), [0x02, b'h', b'i']);

    // Formats as "x" when measured, then "xx" when written
    let mut buf = [0u8; 8];
    assert!(matches!(
        to_slice(&Growing(core::cell::Cell::new(1)), &mut buf),
        Err(SerializerError::Custom(_))
    ));
}
