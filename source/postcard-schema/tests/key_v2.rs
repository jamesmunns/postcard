//! Version 2 keys, checked against the keys that the published postcard-schema
//! 0.2.6 and postcard-schema-ng 0.4.1 crates compute for the same types, each
//! type deriving `Schema` with each crate.

use postcard_schema::{
    key::{v2::Flavor, Item, Key, Options, OwnedItem},
    schema::OwnedDataModelType,
    Schema,
};
use postcard_schema_ng_v0_4 as ng_crate;
use postcard_schema_v0_2 as v0_2_crate;
use std::collections::{BTreeMap, BTreeSet, HashMap};

const V0_2: Flavor = Flavor::PostcardSchemaV0_2;
const NG: Flavor = Flavor::PostcardSchemaNgV0_4;

/// Defines each type three times: in `new`, deriving this crate's `Schema`; in
/// `v0_2`, deriving postcard-schema 0.2's; and in `ng`, deriving
/// postcard-schema-ng 0.4's. Within each module, `Key` is that crate's `Key`.
macro_rules! types {
    ($($item:item)*) => {
        mod new {
            #![allow(dead_code)]
            use super::*;
            use postcard_schema::key::Key;
            $( #[derive(postcard_schema::Schema)] $item )*
        }
        mod v0_2 {
            #![allow(dead_code)]
            use super::*;
            use postcard_schema_v0_2::key::Key;
            $(
                #[derive(postcard_schema_v0_2::Schema)]
                #[postcard(crate = ::postcard_schema_v0_2)]
                $item
            )*
        }
        mod ng {
            #![allow(dead_code)]
            use super::*;
            use postcard_schema_ng_v0_4::key::Key;
            $(
                #[derive(postcard_schema_ng_v0_4::Schema)]
                #[postcard(crate = ::postcard_schema_ng_v0_4)]
                $item
            )*
        }
    };
}

types! {
    pub struct Prims {
        a: u8,
        b: i16,
        c: u32,
        d: i64,
        e: u128,
        f: f32,
        g: f64,
        h: char,
        i: bool,
        j: i8,
        k: u16,
        l: i32,
        m: u64,
        n: i128,
    }

    pub struct Coll {
        a: String,
        b: Vec<u8>,
        c: Option<Vec<u16>>,
        d: (u8, i8),
        e: [u32; 3],
        f: BTreeMap<String, u64>,
        g: (),
        h: HashMap<u8, Vec<String>>,
        i: BTreeSet<i32>,
    }

    pub struct Unit;

    pub struct New(u32);

    pub struct Tup(u8, String);

    pub enum E {
        A,
        B(u8),
        C(u8, u16),
        D { x: bool, y: Option<u8> },
    }

    pub struct Nested {
        e: E,
        n: New,
        t: Tup,
        u: Unit,
        v: Vec<E>,
        o: Option<Box<Tup>>,
    }

    pub enum Empty {}

    pub struct Res {
        a: Result<u8, String>,
        b: Option<Result<(), E>>,
    }

    #[cfg(all(
        feature = "heapless-v0_8",
        feature = "nalgebra-v0_33",
        feature = "chrono-v0_4",
        feature = "fixed-v1_0",
        feature = "uuid-v1_0",
        feature = "serde-big-array-v0_5"
    ))]
    pub struct Feat {
        a: heapless_v0_8::Vec<u8, 4>,
        b: heapless_v0_8::String<8>,
        c: nalgebra_v0_33::Matrix2x3<f32>,
        e: chrono_v0_4::DateTime<chrono_v0_4::Utc>,
        f: fixed_v1_0::types::I16F16,
        g: uuid_v1_0::Uuid,
        h: serde_big_array_v0_5::Array<u8, 40>,
        i: core::num::Saturating<u16>,
    }

    pub enum TopicDirection {
        ToServer,
        ToClient,
    }

    /// postcard-rpc 0.12's schema data message, generic over the schema type it
    /// holds
    pub enum OwnedSchemaData<T> {
        Type(T),
        Endpoint {
            path: String,
            request_key: Key,
            response_key: Key,
        },
        Topic {
            path: String,
            key: Key,
            direction: TopicDirection,
        },
    }
}

/// This crate's version 2 key of `T` at `path`, after checking that the owned
/// hasher gives the same key
#[track_caller]
fn key_v2<T: Schema + ?Sized>(flavor: Flavor, path: &str) -> [u8; 8] {
    let key = Key::new_v2(
        flavor,
        &Options::DEFAULT,
        &[Item::String(path), Item::Schema(T::SCHEMA)],
    );
    let owned: OwnedDataModelType = T::SCHEMA.into();
    let owned_key = Key::new_owned_v2(
        flavor,
        &Options::DEFAULT,
        &[OwnedItem::String(path), OwnedItem::Schema(&owned)],
    );
    assert_eq!(key, owned_key, "{path} {flavor:?} (owned)");
    key.to_bytes()
}

fn key_v0_2<T: v0_2_crate::Schema + ?Sized>(path: &str) -> [u8; 8] {
    v0_2_crate::key::Key::for_path::<T>(path).to_bytes()
}

fn key_ng<T: ng_crate::Schema + ?Sized>(path: &str) -> [u8; 8] {
    ng_crate::key::Key::for_path::<T>(path).to_bytes()
}

/// Checks this crate's version 2 keys of a type against those of
/// postcard-schema 0.2 and postcard-schema-ng 0.4: `check!(path, T)` for a type
/// all three crates implement `Schema` for, or `check!(path, derived T)` for a
/// type defined with `types!`.
macro_rules! check {
    ($path:expr, derived $t:ident) => {
        check!(@ $path, new::$t, v0_2::$t, ng::$t)
    };
    ($path:expr, $t:ty) => {
        check!(@ $path, $t, $t, $t)
    };
    (@ $path:expr, $new:ty, $v0_2:ty, $ng:ty) => {{
        let path = $path;
        assert_eq!(key_v2::<$new>(V0_2, path), key_v0_2::<$v0_2>(path), "{path} 0.2");
        assert_eq!(key_v2::<$new>(NG, path), key_ng::<$ng>(path), "{path} ng");
    }};
}

#[test]
fn matches_published() {
    check!("u8", u8);
    check!("string", String);
    check!("vec_u8", Vec<u8>);
    check!("opt", Option<u32>);
    check!("tuple", (u8, u16, i64));
    check!("array", [u8; 8]);
    check!("duration", std::time::Duration);
    check!("prims", derived Prims);
    check!("coll", derived Coll);
    check!("unitst", derived Unit);
    check!("newtype", derived New);
    check!("tupst", derived Tup);
    check!("enum", derived E);
    check!("nested", derived Nested);
    check!("", derived Nested);
    check!("empty", derived Empty);
}

/// postcard-schema 0.2 described `Ok` and `Err` as tuple variants of one field,
/// and postcard-schema-ng 0.4 as newtype variants, so the flavors give
/// different keys.
#[test]
fn result_differs_by_flavor() {
    check!("result", derived Res);
    assert_ne!(
        key_v2::<new::Res>(V0_2, "result"),
        key_v2::<new::Res>(NG, "result")
    );
}

/// The impls for other crates' types
#[cfg(all(
    feature = "heapless-v0_8",
    feature = "nalgebra-v0_33",
    feature = "chrono-v0_4",
    feature = "fixed-v1_0",
    feature = "uuid-v1_0",
    feature = "serde-big-array-v0_5"
))]
#[test]
fn matches_published_features() {
    check!("features", derived Feat);
}

/// postcard-rpc 0.12's schema data topic holds a schema, which the older crates
/// described with the `Schema` kind. This version has no such kind, so
/// converting it fails.
///
/// The `Schema` kind is found wherever it is, so converting any type that
/// contains it fails.
#[cfg(feature = "postcard-schema-v0_2")]
#[test]
fn schema_kind_unsupported_v0_2() {
    use postcard_schema::legacy::{
        v0_2::{
            OwnedDataModelType as Ty, OwnedDataModelVariant as Var, OwnedNamedType,
            OwnedNamedValue, OwnedNamedVariant,
        },
        SchemaKindUnsupported,
    };
    type Data = self::v0_2::OwnedSchemaData<OwnedNamedType>;
    let nt: OwnedNamedType = <Data as v0_2_crate::Schema>::SCHEMA.into();
    assert_eq!(
        OwnedDataModelType::try_from(&nt),
        Err(SchemaKindUnsupported)
    );

    let named = |ty| OwnedNamedType {
        name: "t".into(),
        ty,
    };
    let schema = || named(Ty::Schema);
    let u8 = || named(Ty::U8);
    let field = || OwnedNamedValue {
        name: "f".into(),
        ty: schema(),
    };
    let variant = |ty| {
        Ty::Enum(vec![OwnedNamedVariant {
            name: "v".into(),
            ty,
        }])
    };
    let positions = [
        Ty::Schema,
        Ty::Option(Box::new(schema())),
        Ty::NewtypeStruct(Box::new(schema())),
        Ty::Seq(Box::new(schema())),
        Ty::Tuple(vec![u8(), schema()]),
        Ty::TupleStruct(vec![u8(), schema()]),
        Ty::Map {
            key: Box::new(schema()),
            val: Box::new(u8()),
        },
        Ty::Map {
            key: Box::new(u8()),
            val: Box::new(schema()),
        },
        Ty::Struct(vec![field()]),
        variant(Var::NewtypeVariant(Box::new(schema()))),
        variant(Var::TupleVariant(vec![u8(), schema()])),
        variant(Var::StructVariant(vec![field()])),
    ];
    for ty in positions {
        let nt = named(ty);
        assert_eq!(
            OwnedDataModelType::try_from(&nt),
            Err(SchemaKindUnsupported),
            "{nt:?}"
        );
    }
}

/// As [`schema_kind_unsupported_v0_2`], for postcard-schema-ng 0.4.
#[cfg(feature = "postcard-schema-ng-v0_4")]
#[test]
fn schema_kind_unsupported_ng_v0_4() {
    use postcard_schema::legacy::{
        ng_v0_4::{OwnedData, OwnedDataModelType as Ty, OwnedNamedField, OwnedVariant},
        SchemaKindUnsupported,
    };
    type Data = ng::OwnedSchemaData<Ty>;
    let ty: Ty = <Data as ng_crate::Schema>::SCHEMA.into();
    assert_eq!(
        OwnedDataModelType::try_from(&ty),
        Err(SchemaKindUnsupported)
    );

    let schema = || Box::new(Ty::Schema);
    let fields = || {
        OwnedData::Struct(Box::new([OwnedNamedField {
            name: "f".into(),
            ty: Ty::Schema,
        }]))
    };
    let datas = || {
        [
            OwnedData::Newtype(schema()),
            OwnedData::Tuple(Box::new([Ty::U8, Ty::Schema])),
            fields(),
        ]
    };
    let mut positions = vec![
        Ty::Schema,
        Ty::Option(schema()),
        Ty::Seq {
            element: schema(),
            max_len: None,
        },
        Ty::Tuple(Box::new([Ty::U8, Ty::Schema])),
        Ty::Map {
            key: schema(),
            val: Box::new(Ty::U8),
            max_len: None,
        },
        Ty::Map {
            key: Box::new(Ty::U8),
            val: schema(),
            max_len: None,
        },
    ];
    for data in datas() {
        positions.push(Ty::Struct {
            name: "s".into(),
            data,
        });
    }
    for data in datas() {
        positions.push(Ty::Enum {
            name: "e".into(),
            variants: Box::new([OwnedVariant {
                name: "v".into(),
                data,
            }]),
        });
    }
    for ty in positions {
        assert_eq!(
            OwnedDataModelType::try_from(&ty),
            Err(SchemaKindUnsupported),
            "{ty:?}"
        );
    }
}

/// The kinds that this version doesn't have become the kinds with the same
/// encoding, and maximum lengths are kept where the older crate had them.
#[cfg(feature = "postcard-schema-v0_2")]
#[test]
fn legacy_kinds_convert_v0_2() {
    use postcard_schema::legacy::v0_2;
    use OwnedDataModelType as T;
    let v0_2 = |ty| {
        let nt = v0_2::OwnedNamedType {
            name: "t".into(),
            ty,
        };
        OwnedDataModelType::try_from(&nt).unwrap()
    };
    assert_eq!(v0_2(v0_2::OwnedDataModelType::Usize), T::U64);
    assert_eq!(v0_2(v0_2::OwnedDataModelType::Isize), T::I64);
    assert_eq!(
        v0_2(v0_2::OwnedDataModelType::ByteArray),
        T::ByteArray { max_len: None }
    );
}

/// As [`legacy_kinds_convert_v0_2`], for postcard-schema-ng 0.4.
#[cfg(feature = "postcard-schema-ng-v0_4")]
#[test]
fn legacy_kinds_convert_ng_v0_4() {
    use postcard_schema::legacy::ng_v0_4::OwnedDataModelType as Ng;
    use OwnedDataModelType as T;
    let ng = |ty: Ng| OwnedDataModelType::try_from(&ty).unwrap();
    assert_eq!(ng(Ng::Usize), T::U64);
    assert_eq!(ng(Ng::Isize), T::I64);
    assert_eq!(
        ng(Ng::ByteArray { max_len: Some(3) }),
        T::ByteArray { max_len: Some(3) }
    );
    assert_eq!(
        ng(Ng::Seq {
            element: Box::new(Ng::U8),
            max_len: Some(4)
        }),
        T::Seq {
            element: Box::new(T::U8),
            max_len: Some(4)
        }
    );
    assert_eq!(
        ng(Ng::Map {
            key: Box::new(Ng::U8),
            val: Box::new(Ng::Bool),
            max_len: Some(5)
        }),
        T::Map {
            key: Box::new(T::U8),
            val: Box::new(T::Bool),
            max_len: Some(5)
        }
    );
}

/// These keys can't be matched: postcard-schema 0.5 describes these types as
/// serde writes them, and the older crates did not.
#[test]
fn net_types_differ() {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};
    macro_rules! differs {
        ($($t:ty),*) => {$(
            let path = stringify!($t);
            assert_ne!(key_v2::<$t>(V0_2, path), key_v0_2::<$t>(path), "{path} 0.2");
            assert_ne!(key_v2::<$t>(NG, path), key_ng::<$t>(path), "{path} ng");
        )*};
    }
    differs!(Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6);
}

/// The maximum lengths option matches postcard-schema-ng's `hash_max_size`, and
/// is ignored for postcard-schema 0.2, which had no maximum lengths.
#[test]
fn max_len_matches_ng() {
    use ng_crate::key::hash::{fnv1a64::hasher, HashItem, HashOptions};
    type S = postcard_schema::max_len::MaxLenString<32>;
    type NgS = ng_crate::max_len::MaxLenString<32>;

    let items = [
        Item::String("m"),
        Item::Schema(<(S, Vec<u8>, Option<S>)>::SCHEMA),
    ];
    let opt = Options::DEFAULT.with_max_len(true);
    let mut ng_opt = HashOptions::DEFAULT;
    ng_opt.hash_max_size = true;
    let ng = hasher(
        &ng_opt,
        &[
            HashItem::String("m"),
            HashItem::Schema(<(NgS, Vec<u8>, Option<NgS>) as ng_crate::Schema>::SCHEMA),
        ],
    );
    assert_eq!(Key::new_v2(NG, &opt, &items).to_bytes(), ng);
    assert_eq!(
        Key::new_v2(V0_2, &opt, &items),
        Key::new_v2(V0_2, &Options::DEFAULT, &items)
    );
}

/// A schema that a postcard-schema 0.2 device sends, once converted, gives the
/// key that postcard-schema 0.2 computes, including for the types that this
/// version describes differently.
#[cfg(feature = "postcard-schema-v0_2")]
#[test]
fn converted_v0_2_schemas_match() {
    use postcard_schema::legacy::v0_2::OwnedNamedType;
    use std::net::{Ipv4Addr, SocketAddr};

    #[track_caller]
    fn converts<T: v0_2_crate::Schema + ?Sized>(path: &str) {
        let nt: OwnedNamedType = T::SCHEMA.into();
        let ty = OwnedDataModelType::try_from(&nt).unwrap();
        let key = Key::new_owned_v2(
            V0_2,
            &Options::DEFAULT,
            &[OwnedItem::String(path), OwnedItem::Schema(&ty)],
        );
        assert_eq!(key.to_bytes(), key_v0_2::<T>(path), "{path}");
    }

    converts::<u8>("u8");
    converts::<String>("string");
    converts::<Vec<u8>>("vec_u8");
    converts::<Option<u32>>("opt");
    converts::<(u8, u16, i64)>("tuple");
    converts::<[u8; 8]>("array");
    converts::<std::time::Duration>("duration");
    converts::<v0_2::Prims>("prims");
    converts::<v0_2::Coll>("coll");
    converts::<v0_2::Unit>("unitst");
    converts::<v0_2::New>("newtype");
    converts::<v0_2::Tup>("tupst");
    converts::<v0_2::E>("enum");
    converts::<v0_2::Nested>("nested");
    converts::<v0_2::Empty>("empty");
    converts::<v0_2::Res>("result");
    converts::<Ipv4Addr>("ipv4");
    converts::<SocketAddr>("sockaddr");
}

/// A schema that a postcard-schema-ng 0.4 device sends, once converted,
/// gives the key that postcard-schema-ng 0.4 computes, with and without
/// maximum lengths (temperance hashes with them).
#[cfg(feature = "postcard-schema-ng-v0_4")]
#[test]
fn converted_ng_v0_4_schemas_match() {
    use ng_crate::key::hash::{fnv1a64::hasher, HashItem, HashOptions};
    use postcard_schema::legacy::ng_v0_4::OwnedDataModelType as NgOwned;

    #[track_caller]
    fn converts<T: ng_crate::Schema + ?Sized>(path: &str) {
        let ng: NgOwned = T::SCHEMA.into();
        let ty = OwnedDataModelType::try_from(&ng).unwrap();
        for max_len in [false, true] {
            let mut ng_opt = HashOptions::DEFAULT;
            ng_opt.hash_max_size = max_len;
            let expected = hasher(
                &ng_opt,
                &[HashItem::String(path), HashItem::Schema(T::SCHEMA)],
            );
            let key = Key::new_owned_v2(
                NG,
                &Options::DEFAULT.with_max_len(max_len),
                &[OwnedItem::String(path), OwnedItem::Schema(&ty)],
            );
            assert_eq!(key.to_bytes(), expected, "{path} max_len={max_len}");
        }
    }

    converts::<u8>("u8");
    converts::<String>("string");
    converts::<ng_crate::max_len::MaxLenString<16>>("maxlen");
    converts::<Vec<u8>>("vec_u8");
    converts::<Option<u32>>("opt");
    converts::<(u8, u16, i64)>("tuple");
    converts::<[u8; 8]>("array");
    converts::<std::time::Duration>("duration");
    converts::<ng::Prims>("prims");
    converts::<ng::Coll>("coll");
    converts::<ng::Unit>("unitst");
    converts::<ng::New>("newtype");
    converts::<ng::Tup>("tupst");
    converts::<ng::E>("enum");
    converts::<ng::Nested>("nested");
    converts::<ng::Empty>("empty");
    converts::<ng::Res>("result");
}

/// postcard-derive 0.2.2 and later honour `#[serde(rename)]`, as this crate's
/// derive does, so renamed types match postcard-schema 0.2.6.
/// postcard-derive-ng ignores it, so they can't match postcard-schema-ng 0.4.
#[test]
fn serde_rename() {
    #[derive(serde::Serialize, postcard_schema::Schema)]
    struct New {
        #[serde(rename = "b")]
        a: u8,
    }
    #[derive(serde::Serialize, v0_2_crate::Schema)]
    #[postcard(crate = ::postcard_schema_v0_2)]
    struct OldV0_2 {
        #[serde(rename = "b")]
        a: u8,
    }
    #[derive(serde::Serialize, ng_crate::Schema)]
    #[postcard(crate = ::postcard_schema_ng_v0_4)]
    struct OldNg {
        #[serde(rename = "b")]
        a: u8,
    }
    assert_eq!(key_v2::<New>(V0_2, "r"), key_v0_2::<OldV0_2>("r"));
    assert_ne!(key_v2::<New>(NG, "r"), key_ng::<OldNg>("r"));
}
