//! Checks every `Schema` impl in this crate against its serde impls, with
//! `postcard_schema::verify`.
//!
//! Each impl needs sample values here that together reach every part of its
//! schema. `Report::check` and `Report::check_all` record a failure if they
//! don't, and `Report::finish` reports every failure; `Report::check_ser` only
//! checks serializing, without coverage. `ci.sh` runs this with all
//! features, so that every feature-gated impl is checked.

use core::num::{NonZeroI32, NonZeroU16, NonZeroUsize, Wrapping};
use postcard_schema::{
    key::Key,
    max_len::{MaxLenByteSlice, MaxLenBytes, MaxLenStr, MaxLenString},
    verify::{verify_all, verify_serialize, Error},
    Schema,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::PathBuf,
    time::Duration,
};

/// Collects failures, so one run reports every impl that disagrees
#[derive(Default)]
struct Report(Vec<String>);

impl Report {
    /// Checks one value, which must reach every part of the schema
    #[track_caller]
    fn check<T>(&mut self, what: &str, value: T)
    where
        T: Schema + Serialize + DeserializeOwned,
    {
        self.check_all(what, &[value]);
    }

    /// Checks values that together reach every part of the schema
    #[track_caller]
    fn check_all<T>(&mut self, what: &str, values: &[T])
    where
        T: Schema + Serialize + DeserializeOwned,
    {
        if let Err(e) = verify_all(values) {
            self.fail(what, e);
        }
    }

    /// Only the serializing half, for types that can't be deserialized, or that
    /// borrow
    #[track_caller]
    fn check_ser<T>(&mut self, what: &str, value: &T)
    where
        T: Schema + Serialize + ?Sized,
    {
        if let Err(e) = verify_serialize(value) {
            self.fail(what, e);
        }
    }

    fn fail(&mut self, what: &str, e: Error) {
        self.0.push(format!("{what}: {e}"));
    }

    #[track_caller]
    fn finish(self) {
        assert!(
            self.0.is_empty(),
            "{} failures:\n{}",
            self.0.len(),
            self.0.join("\n")
        );
    }
}

#[test]
fn core_types() {
    let mut r = Report::default();
    r.check("bool", true);
    r.check("i8", -1i8);
    r.check("i16", -300i16);
    r.check("i32", i32::MIN);
    r.check("i64", i64::MIN);
    r.check("i128", i128::MIN);
    r.check("u8", 255u8);
    r.check("u16", 300u16);
    r.check("u32", u32::MAX);
    r.check("u64", u64::MAX);
    r.check("u128", u128::MAX);
    r.check("usize", usize::MAX);
    r.check("isize", isize::MIN);
    r.check("NonZeroU16", NonZeroU16::new(7).unwrap());
    r.check("NonZeroI32", NonZeroI32::new(-7).unwrap());
    r.check("NonZeroUsize", NonZeroUsize::new(7).unwrap());
    r.check("f32", f32::NAN);
    r.check("f64", -0.0f64);
    r.check("char", '⍺');
    r.check("()", ());
    r.check_all("Option", &[None, Some(3u8)]);
    r.check_all("Result", &[Ok::<u8, String>(1), Err("no".into())]);
    r.check("tuple", (1u8, -2i64, "three".to_string(), 'f'));
    r.check("[u8; 4]", [1u8, 2, 3, 4]);
    r.check("[u8; 0]", [0u8; 0]);
    r.check("Range", 1u32..5);
    r.check("RangeInclusive", 1u32..=5);
    r.check("RangeFrom", 1u32..);
    r.check("RangeTo", ..5u32);
    r.check("Ipv4Addr", Ipv4Addr::new(10, 0, 0, 1));
    r.check("Ipv6Addr", Ipv6Addr::LOCALHOST);
    r.check_all(
        "IpAddr",
        &[
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
        ],
    );
    r.check("SocketAddrV4", SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80));
    r.check(
        "SocketAddrV6",
        SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 0, 0),
    );
    r.check_all(
        "SocketAddr",
        &[
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80)),
            SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 0, 0)),
        ],
    );
    r.check("Wrapping", Wrapping(5u16));
    r.check("Saturating", core::num::Saturating(5u16));
    r.check("Duration", Duration::new(5, 6));
    r.check("PhantomData", PhantomData::<u8>);
    r.check_ser("Arguments", &format_args!("{}", 5));
    r.check_ser("&str", &"hi");
    r.check_ser("[u16]", &[1u16, 2][..]);
    r.check("Key", Key::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]));
    r.finish();
}

#[test]
fn std_types() {
    let mut r = Report::default();
    r.check_all("Vec", &[vec![1u16, 2, 3], vec![]]);
    r.check("String", "hello".to_string());
    r.check("PathBuf", PathBuf::from("/a/b"));
    r.check("HashMap", HashMap::from([(1u8, "a".to_string())]));
    r.check("BTreeMap", BTreeMap::from([(1u8, 2u32), (3, 4)]));
    r.check("HashSet", HashSet::from([1u8]));
    r.check("BTreeSet", BTreeSet::from([1u8, 2]));
    r.check("Box", Box::new(5u8));
    r.check("Cow<str>", Cow::<str>::Owned("a".into()));
    r.finish();
}

#[test]
fn max_len_types() {
    let mut r = Report::default();
    r.check("MaxLenString", MaxLenString::<8>::try_from("hi").unwrap());
    r.check(
        "MaxLenBytes",
        MaxLenBytes::<8>::try_from(&[1u8, 2][..]).unwrap(),
    );
    let s = MaxLenStr::<8>::try_from("hi").unwrap();
    match verify_serialize(&s) {
        Ok(rec) => {
            if let Err(e) = rec.verify_deserialize::<MaxLenStr<'_, 8>>() {
                r.fail("MaxLenStr", e);
            }
        }
        Err(e) => r.fail("MaxLenStr", e),
    }
    let b = MaxLenByteSlice::<8>::try_from(&[1u8, 2][..]).unwrap();
    match verify_serialize(&b) {
        Ok(rec) => {
            if let Err(e) = rec.verify_deserialize::<MaxLenByteSlice<'_, 8>>() {
                r.fail("MaxLenByteSlice", e);
            }
        }
        Err(e) => r.fail("MaxLenByteSlice", e),
    }
    r.finish();
}

#[test]
fn feature_types() {
    // every check here is feature-gated
    #[allow(unused_mut)]
    let mut r = Report::default();

    #[cfg(feature = "chrono-v0_4")]
    {
        use chrono_v0_4::{DateTime, Utc};
        r.check(
            "chrono DateTime<Utc>",
            DateTime::<Utc>::from_timestamp(5, 6).unwrap(),
        );
    }
    //
    // TODO: EnumMap isn't checked: enum-map 3 needs rustc 1.95, so it can't be
    // a dev-dependency (with its serde feature) under the 1.88 MSRV.
    //
    #[cfg(feature = "fixed-v1_0")]
    {
        use fixed_v1_0::types::{I16F16, U8F8};
        r.check("FixedI32", I16F16::from_num(1.5));
        r.check("FixedU16", U8F8::from_num(1.5));
    }
    #[cfg(feature = "heapless-v0_7")]
    r.check(
        "heapless 0.7 Vec",
        heapless_v0_7::Vec::<u8, 4>::from_slice(&[1, 2]).unwrap(),
    );
    #[cfg(feature = "heapless-v0_7")]
    r.check(
        "heapless 0.7 String",
        heapless_v0_7::String::<4>::from("hi"),
    );
    #[cfg(feature = "heapless-v0_8")]
    r.check(
        "heapless 0.8 Vec",
        heapless_v0_8::Vec::<u8, 4>::from_slice(&[1, 2]).unwrap(),
    );
    #[cfg(feature = "heapless-v0_8")]
    r.check(
        "heapless 0.8 String",
        heapless_v0_8::String::<4>::try_from("hi").unwrap(),
    );
    #[cfg(feature = "heapless-v0_9")]
    r.check(
        "heapless 0.9 Vec",
        heapless_v0_9::Vec::<u8, 4>::from_slice(&[1, 2]).unwrap(),
    );
    #[cfg(feature = "heapless-v0_9")]
    r.check(
        "heapless 0.9 String",
        heapless_v0_9::String::<4>::try_from("hi").unwrap(),
    );
    #[cfg(feature = "nalgebra-v0_33")]
    {
        use nalgebra_v0_33 as na;
        r.check("na 0.33 Matrix2x3", na::Matrix2x3::new(1u8, 2, 3, 4, 5, 6));
        r.check("na 0.33 Point3", na::Point3::new(1u8, 2, 3));
        r.check(
            "na 0.33 Quaternion",
            na::Quaternion::new(1.0f32, 2.0, 3.0, 4.0),
        );
        r.check(
            "na 0.33 UnitQuaternion",
            na::Unit::new_unchecked(na::Quaternion::new(1.0f32, 0.0, 0.0, 0.0)),
        );
    }
    #[cfg(feature = "nalgebra-v0_34")]
    {
        use nalgebra_v0_34 as na;
        r.check("na 0.34 Matrix2x3", na::Matrix2x3::new(1u8, 2, 3, 4, 5, 6));
        r.check("na 0.34 Point3", na::Point3::new(1u8, 2, 3));
        r.check(
            "na 0.34 Quaternion",
            na::Quaternion::new(1.0f32, 2.0, 3.0, 4.0),
        );
        r.check(
            "na 0.34 UnitQuaternion",
            na::Unit::new_unchecked(na::Quaternion::new(1.0f32, 0.0, 0.0, 0.0)),
        );
    }
    #[cfg(feature = "serde-big-array-v0_5")]
    r.check(
        "serde_big_array Array",
        serde_big_array_v0_5::Array([7u8; 40]),
    );
    #[cfg(feature = "uuid-v1_0")]
    r.check("Uuid", uuid_v1_0::Uuid::from_u128(0x1234));
    r.finish();
}

#[allow(dead_code)]
#[derive(Schema, Serialize, Deserialize)]
#[serde(rename = "Renamed")]
struct Derived {
    a: u8,
    #[serde(rename = "bee")]
    b: Option<String>,
    c: Vec<Inner>,
    d: (u8, Inner),
    e: Unit,
    f: Newtype,
    g: TupleStruct,
}

#[derive(Schema, Serialize, Deserialize)]
enum Inner {
    A,
    #[serde(rename = "Bee")]
    B(u8),
    C(u8, i16),
    D {
        x: bool,
    },
}

#[derive(Schema, Serialize, Deserialize)]
struct Unit;

#[derive(Schema, Serialize, Deserialize)]
struct Newtype(u32);

#[derive(Schema, Serialize, Deserialize)]
struct TupleStruct(u8, i8);

#[test]
fn derived_types() {
    let mut r = Report::default();
    let derived = |b| Derived {
        a: 1,
        b,
        c: vec![Inner::A, Inner::B(2), Inner::C(3, -4), Inner::D { x: true }],
        d: (5, Inner::A),
        e: Unit,
        f: Newtype(6),
        g: TupleStruct(7, -8),
    };
    r.check_all("Derived", &[derived(Some("x".into())), derived(None)]);
    r.finish();
}

/// The schema table types' hand-written `Schema` impls.
#[test]
fn schema_table() {
    use postcard_schema::{
        table::{
            node::{NameRef, Span, TableData as D, TableField, TableNode as N, TableVariant},
            RawOwnedSchemaTable, SchemaTableRef,
        },
        verify::Coverage,
    };
    let name = NameRef { offset: 0, len: 1 };
    let span = Span { start: 0, len: 1 };
    let nodes = [
        N::Bool,
        N::I8,
        N::U8,
        N::I16,
        N::I32,
        N::I64,
        N::I128,
        N::U16,
        N::U32,
        N::U64,
        N::U128,
        N::F32,
        N::F64,
        N::Char,
        N::String { max_len: None },
        N::String { max_len: Some(3) },
        N::ByteArray { max_len: None },
        N::Option(0),
        N::Unit,
        N::Seq {
            element: 0,
            max_len: None,
        },
        N::Tuple(span),
        N::Map {
            key: 0,
            val: 1,
            max_len: None,
        },
        N::Struct {
            name,
            data: D::Unit,
        },
        N::Struct {
            name,
            data: D::Newtype(0),
        },
        N::Struct {
            name,
            data: D::Tuple(span),
        },
        N::Enum {
            name,
            variants: span,
        },
        N::Array { element: 0, len: 4 },
    ];
    let variants = [TableVariant {
        name,
        data: D::Struct(span),
    }];
    let fields = [TableField { name, ty: 0 }];
    let table = SchemaTableRef {
        version: 1,
        roots: &[0],
        nodes: &nodes,
        children: &[0],
        variants: &variants,
        fields: &fields,
        names: "a",
    };
    let mut coverage = Coverage::new::<SchemaTableRef>();
    coverage.add(&verify_serialize(&table).unwrap()).unwrap();
    coverage.check().unwrap();
    verify_all(&[RawOwnedSchemaTable::from(table)]).unwrap();
}
