//! Tests of `postcard_schema::verify` itself: schemas that disagree with their
//! serde impls must be reported.

#![allow(dead_code)]

use postcard_schema::{
    max_len::MaxLenString,
    schema::{Data, DataModelType, NamedField},
    verify::{verify, verify_all, verify_serialize, Coverage, Error},
    Schema,
};
use serde::{de::DeserializeOwned, de::SeqAccess, Deserialize, Deserializer, Serialize};

#[track_caller]
fn fails<T: Schema + Serialize + DeserializeOwned>(value: T, path: &str, msg: &str) {
    let e: Error = verify(&value).expect_err("verify should fail");
    assert_eq!(e.path(), path, "{e}");
    assert!(e.message().contains(msg), "{e}");
}

#[derive(Schema, Serialize, Deserialize)]
struct Good {
    a: u8,
    b: Vec<Option<(u16, String)>>,
    c: Choice,
}

#[derive(Schema, Serialize, Deserialize)]
enum Choice {
    A,
    B(i8),
    C { x: bool },
}

#[test]
fn accepts_matching() {
    verify(&Good {
        a: 1,
        b: vec![None, Some((2, "x".into()))],
        c: Choice::C { x: true },
    })
    .unwrap();
    verify(&Choice::B(-1)).unwrap();
    verify(&Choice::A).unwrap();
}

/// A `u16` that claims to be a `u32`
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct WrongPrim(u16);

impl Schema for WrongPrim {
    const SCHEMA: &'static DataModelType = &DataModelType::U32;
}

#[test]
fn wrong_kind() {
    fails(
        WrongPrim(1),
        "",
        "`serialize_u16` was called, but the schema is U32",
    );
}

#[derive(Schema, Serialize, Deserialize)]
struct Outer {
    list: Vec<Inner>,
}

#[derive(Schema, Serialize, Deserialize)]
struct Inner {
    x: WrongPrim,
}

#[test]
fn reports_path() {
    fails(
        Outer {
            list: vec![Inner { x: WrongPrim(1) }],
        },
        ".list[0].x",
        "`serialize_u16`",
    );
}

/// Serializes with `serialize_bytes`, but claims to be a `Seq` of `u8`
#[derive(Serialize, Deserialize)]
struct Bytes(#[serde(with = "as_bytes")] Vec<u8>);

mod as_bytes {
    pub fn serialize<S: serde::Serializer>(v: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        serde::Deserialize::deserialize(d)
    }
}

impl Schema for Bytes {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "Bytes",
        data: Data::Newtype(<Vec<u8>>::SCHEMA),
    };
}

#[test]
fn bytes_are_not_a_seq() {
    fails(
        Bytes(vec![1]),
        ".0",
        "`serialize_bytes` was called, but the schema is Seq",
    );
}

/// A field name that doesn't match serde's
#[derive(Serialize, Deserialize)]
struct Named {
    a: u8,
}

impl Schema for Named {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "Named",
        data: Data::Struct(&[&NamedField {
            name: "b",
            ty: u8::SCHEMA,
        }]),
    };
}

#[test]
fn wrong_field_name() {
    fails(
        Named { a: 1 },
        "",
        "serde's field name is `a`, but the schema's is `b`",
    );
}

/// A string longer than its schema's `max_len`
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct Long(String);

impl Schema for Long {
    const SCHEMA: &'static DataModelType = <MaxLenString<2>>::SCHEMA;
}

#[test]
fn max_len() {
    fails(
        Long("abc".into()),
        "",
        "length 3 is more than the schema's max_len of 2",
    );
}

#[derive(Schema, Serialize, Deserialize)]
struct Skips {
    a: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    b: Option<u8>,
}

#[test]
fn skipped_field() {
    fails(
        Skips { a: 1, b: None },
        "",
        "serde has 1 struct fields, but the schema has 2",
    );
}

/// Serializes two elements, but its deserializer only takes the first
#[derive(Serialize)]
struct TakesOne(u8, u8);

impl<'de> Deserialize<'de> for TakesOne {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = TakesOne;
            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str("a tuple struct")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<TakesOne, A::Error> {
                let a = seq.next_element()?.unwrap_or_default();
                Ok(TakesOne(a, a))
            }
        }
        d.deserialize_tuple_struct("TakesOne", 2, V)
    }
}

impl Schema for TakesOne {
    const SCHEMA: &'static DataModelType = &DataModelType::Struct {
        name: "TakesOne",
        data: Data::Tuple(&[u8::SCHEMA, u8::SCHEMA]),
    };
}

#[test]
fn visitor_takes_every_element() {
    fails(TakesOne(1, 2), "", "the visitor took 1 of 2 elements");
}

/// Deserializes to a value that serializes differently
#[derive(Serialize)]
#[serde(transparent)]
struct Lossy(u8);

impl<'de> Deserialize<'de> for Lossy {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        u8::deserialize(d).map(|v| Lossy(v.wrapping_add(1)))
    }
}

impl Schema for Lossy {
    const SCHEMA: &'static DataModelType = u8::SCHEMA;
}

#[test]
fn round_trip() {
    fails(
        Lossy(1),
        "",
        "serializing the deserialized value gave U8(2), not U8(1)",
    );
}

#[track_caller]
fn uncovered<T: Schema + Serialize + DeserializeOwned>(values: &[T], path: &str, msg: &str) {
    let e: Error = verify_all(values).expect_err("verify_all should fail");
    assert_eq!(e.path(), path, "{e}");
    assert!(e.message().contains(msg), "{e}");
}

/// Values that reach every variant, `None` and `Some`, and a non-empty sequence
#[test]
fn coverage_complete() {
    verify_all(&[
        Good {
            a: 1,
            b: vec![None, Some((2, "x".into()))],
            c: Choice::A,
        },
        Good {
            a: 1,
            b: vec![],
            c: Choice::B(3),
        },
        Good {
            a: 1,
            b: vec![],
            c: Choice::C { x: true },
        },
    ])
    .unwrap();
    verify_all::<std::collections::BTreeMap<u8, Option<u8>>>(&[[(1, None), (2, Some(3))]
        .into_iter()
        .collect()])
    .unwrap();
}

#[test]
fn coverage_reports_first_unreached_part() {
    let good = |b: Vec<Option<(u16, String)>>, c| Good { a: 1, b, c };
    uncovered::<Good>(&[], ".b", "a non-empty sequence");
    uncovered(&[good(vec![], Choice::A)], ".b", "a non-empty sequence");
    uncovered(&[good(vec![None], Choice::A)], ".b[]", "`Some`");
    uncovered(
        &[good(vec![Some((1, "x".into()))], Choice::A)],
        ".b[]",
        "`None`",
    );
    uncovered(
        &[good(vec![None, Some((1, "x".into()))], Choice::A)],
        ".c",
        "variant `B`",
    );
    uncovered(
        &[std::collections::BTreeMap::<u8, u8>::new()],
        "",
        "a non-empty map",
    );
    uncovered(&[Some(None::<u8>), None], ".some", "`Some`");
}

/// A value that fails [`verify`] is reported with its index
#[test]
fn coverage_reports_failing_value() {
    #[derive(Serialize, Deserialize)]
    struct Wrong(u8);
    impl Schema for Wrong {
        const SCHEMA: &'static DataModelType = &DataModelType::U16;
    }
    let e = verify_all(&[Wrong(1)]).unwrap_err();
    assert!(e.message().starts_with("value 0: "), "{e}");
}

/// [`Coverage`] also takes recordings of borrowing types
#[test]
fn coverage_of_recordings() {
    let mut c = Coverage::new::<Option<&str>>();
    c.add(&verify_serialize(&Some("hi")).unwrap()).unwrap();
    assert!(c.check().is_err());
    c.add(&verify_serialize(&None::<&str>).unwrap()).unwrap();
    c.check().unwrap();
    assert!(c.add(&verify_serialize(&3u8).unwrap()).is_err());
}

/// A type only needs to be covered once, across all the places it appears
#[test]
fn coverage_is_per_type() {
    verify_all(&[
        (Choice::A, Choice::B(1)),
        (Choice::A, Choice::C { x: true }),
    ])
    .unwrap();
    uncovered(&[(Choice::A, Choice::B(1))], ".0", "variant `C`");
}

/// Coverage reaches inside map keys and values, tuple structs, and each kind of
/// enum variant, and reports the path of what is missing
#[test]
fn coverage_inside_containers() {
    use std::collections::BTreeMap;

    #[derive(Schema, Serialize, Deserialize)]
    struct Pair(u8, Option<u8>);

    #[derive(Schema, Serialize, Deserialize, Clone)]
    enum Var {
        Newtype(Option<i8>),
        Tuple(Option<u8>, u8),
        Struct { x: Option<u16> },
    }

    uncovered(&[BTreeMap::from([(None::<u8>, 1u8)])], "[].key", "`Some`");
    uncovered(&[BTreeMap::from([(1u8, None::<u8>)])], "[].value", "`Some`");
    verify_all(&[BTreeMap::from([
        (None::<u8>, None::<u8>),
        (Some(1), Some(2)),
    ])])
    .unwrap();

    uncovered(&[Pair(1, None)], ".1", "`Some`");
    verify_all(&[Pair(1, None), Pair(2, Some(3))]).unwrap();

    let all = [
        Var::Newtype(None),
        Var::Newtype(Some(-1)),
        Var::Tuple(None, 1),
        Var::Tuple(Some(2), 3),
        Var::Struct { x: None },
        Var::Struct { x: Some(4) },
    ];
    verify_all(&all).unwrap();
    uncovered(&all[1..], "::Newtype.0", "`None`");
    uncovered(&[&all[..2], &all[3..]].concat(), "::Tuple.0", "`None`");
    uncovered(&all[..5], "::Struct.x", "`Some`");
}
