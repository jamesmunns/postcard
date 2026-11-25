#![allow(dead_code)]

use super::{hasher, owned, Item, Options, State, BASIS};
use crate::{
    key::Key,
    max_len::{MaxLenBytes, MaxLenString},
    schema::{DataModelType, OwnedDataModelType},
    Schema,
};
use model::{el, Item as In, KeyInput, Name, Type};
use std::collections::BTreeMap;

const NO_NAMES: Options = Options::DEFAULT.with_names(false);
const MAX_LEN: Options = Options::DEFAULT.with_max_len(true);

/// The key of a raw input
fn key_for_slice(input: &[u8]) -> [u8; 8] {
    State(BASIS).bytes(input).finish()
}

fn key_with<T: Schema + ?Sized>(opt: &Options) -> [u8; 8] {
    hasher(opt, &[Item::Schema(T::SCHEMA)])
}

fn key<T: Schema + ?Sized>() -> [u8; 8] {
    key_with::<T>(&Options::DEFAULT)
}

/// The key of a path and one schema with the default options
fn path_key<T: Schema + ?Sized>(path: &str) -> [u8; 8] {
    hasher(
        &Options::DEFAULT,
        &[Item::String(path), Item::Schema(T::SCHEMA)],
    )
}

fn method<Q: Schema, R: Schema>(name: &str) -> [u8; 8] {
    hasher(
        &MAX_LEN,
        &[
            Item::String(name),
            Item::Schema(Q::SCHEMA),
            Item::Schema(R::SCHEMA),
        ],
    )
}

/// The spec's `KeyInput` type, which postcard encodes as the expected key input
///
/// `N` and `M` are the spec's `Name` and `MaxLen`: [`Name`] and `Option<u64>`
/// when the option is enabled, and `()` when it is disabled.
mod model {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct KeyInput<N, M> {
        domain: [u8; 22],
        options: u64,
        items: Vec<Item<N, M>>,
    }

    #[derive(Serialize)]
    pub enum Item<N, M> {
        String(&'static str),
        Schema(Type<N, M>),
    }

    /// The variants are in the order of their tags, `0x00` to `0x14`
    #[derive(Serialize, Debug)]
    pub enum Type<N, M> {
        Bool,
        I8,
        U8,
        I16,
        I32,
        I64,
        I128,
        U16,
        U32,
        U64,
        U128,
        F32,
        F64,
        Char,
        String(M),
        Unit,
        Option(Box<Type<N, M>>),
        Sequence(M, Box<Type<N, M>>),
        Aggregate(Vec<Element<N, M>>),
        TaggedUnion(Vec<Element<N, M>>),
        Schema,
    }

    #[derive(Serialize, Debug)]
    pub struct Element<N, M> {
        name: N,
        ty: Type<N, M>,
    }

    /// A name, with the names option enabled
    pub type Name = Option<&'static str>;

    pub fn el<N, M>(name: N, ty: Type<N, M>) -> Element<N, M> {
        Element { name, ty }
    }

    impl<N, M> Type<N, M> {
        pub fn seq(max_len: M, ty: Self) -> Self {
            Type::Sequence(max_len, Box::new(ty))
        }

        pub fn option(ty: Self) -> Self {
            Type::Option(Box::new(ty))
        }
    }

    impl<N: Serialize, M: Serialize> KeyInput<N, M> {
        pub fn new(options: u64, items: Vec<Item<N, M>>) -> Self {
            KeyInput {
                domain: *b"postcard-schema/key/v3",
                options,
                items,
            }
        }

        /// The key of the postcard encoding of this key input
        pub fn key(&self) -> [u8; 8] {
            super::key_for_slice(&postcard::to_vec(self).unwrap())
        }
    }
}

/// Published FNV-1a 128 test vectors
// schema[verify key.hash.fnv1a-128]
#[test]
fn fnv1a_128_vectors() {
    assert_eq!(State(BASIS).0, 0x6c62272e07bb014262b821756295c58d);
    assert_eq!(
        State(BASIS).bytes(b"a").0,
        0xd228cb696f1a8caf78912b704e4a8964
    );
    assert_eq!(
        State(BASIS).bytes(b"foobar").0,
        0x343e1662793c64bf6f0d3597ba446f18
    );
}

/// The fold and `fmix64` of the FNV offset basis (no input bytes)
// schema[verify key.hash.fold]
// schema[verify key.hash.fmix64]
// schema[verify key.hash.output]
#[test]
fn finish() {
    let mut k: u64 = 0x6c62272e07bb0142 ^ 0x62b821756295c58d;
    k ^= k >> 33;
    k = k.wrapping_mul(0xff51afd7ed558ccd);
    k ^= k >> 33;
    k = k.wrapping_mul(0xc4ceb9fe1a85ec53);
    k ^= k >> 33;
    assert_eq!(key_for_slice(&[]), k.to_le_bytes());
}

#[derive(Schema)]
#[postcard(crate = crate)]
struct Ping {
    seq: u32,
    data: Vec<u8>,
}

#[derive(Schema)]
#[postcard(crate = crate)]
enum Cmd {
    Stop,
    Go(u8),
}

/// The key input of a path and a struct with the default options
// schema[verify key.options]
// schema[verify key.input.postcard]
// schema[verify key.input.domain]
// schema[verify key.input.options]
// schema[verify key.input.item]
// schema[verify key.input.name]
// schema[verify key.input.struct]
#[test]
fn input_struct() {
    let input = KeyInput::<Name, ()>::new(
        0x01,
        vec![
            In::String("ping"),
            In::Schema(Type::Aggregate(vec![
                el(Some("seq"), Type::U32),
                el(Some("data"), Type::seq((), Type::U8)),
            ])),
        ],
    );
    assert_eq!(path_key::<Ping>("ping"), input.key());
    assert_eq!(
        path_key::<Ping>("ping"),
        [0x94, 0xa1, 0x8b, 0x70, 0xc2, 0xaa, 0x4b, 0x3f]
    );
}

/// The key input of an enum with the default options
// schema[verify key.input.enum]
#[test]
fn input_enum() {
    let input = KeyInput::<Name, ()>::new(
        0x01,
        vec![
            In::String("cmd"),
            In::Schema(Type::TaggedUnion(vec![
                el(Some("Stop"), Type::Unit),
                el(Some("Go"), Type::U8),
            ])),
        ],
    );
    assert_eq!(path_key::<Cmd>("cmd"), input.key());
    assert_eq!(
        path_key::<Cmd>("cmd"),
        [0x22, 0x11, 0xad, 0x35, 0x85, 0x38, 0xeb, 0x76]
    );
}

/// The key input of a map with maximum lengths and without names
// schema[verify key.input.map]
// schema[verify key.input.max-len]
// schema[verify key.input.options]
// schema[verify key.input.name]
#[test]
fn input_map() {
    let input = KeyInput::<(), Option<u64>>::new(
        0x02,
        vec![In::Schema(Type::seq(
            None,
            Type::Aggregate(vec![el((), Type::String(None)), el((), Type::U8)]),
        ))],
    );
    let opt = Options::DEFAULT.with_names(false).with_max_len(true);
    assert_eq!(key_with::<BTreeMap<String, u8>>(&opt), input.key());
    assert_eq!(
        key_with::<BTreeMap<String, u8>>(&opt),
        [0xc1, 0x19, 0xb9, 0x32, 0x34, 0x91, 0xd6, 0xa0]
    );
}

#[derive(Schema)]
#[postcard(crate = crate)]
struct Read {
    channel: u8,
}

/// The key input of a path, a request, and a response, with names and
/// maximum lengths, as an RPC framework might define a method's key
// schema[verify key.items]
// schema[verify key.item]
// schema[verify key.input.item]
// schema[verify key.input.max-len]
#[test]
fn input_method() {
    let input = KeyInput::<Name, Option<u64>>::new(
        0x03,
        vec![
            In::String("get_temperature_value"),
            In::Schema(Type::Aggregate(vec![el(Some("channel"), Type::U8)])),
            In::Schema(Type::seq(None, Type::String(Some(16)))),
        ],
    );
    let key = method::<Read, Vec<MaxLenString<16>>>("get_temperature_value");
    assert_eq!(key, input.key());
    assert_eq!(key, [0x90, 0x13, 0x41, 0xcc, 0x15, 0xdf, 0xa3, 0xcb]);
}

#[derive(Schema)]
#[postcard(crate = crate)]
struct UnitStruct;
#[derive(Schema)]
#[postcard(crate = crate)]
struct Newtype(u8);
#[derive(Schema)]
#[postcard(crate = crate)]
struct TupleStruct(u8, bool);
#[derive(Schema)]
#[postcard(crate = crate)]
enum Variants {
    Unit,
    Newtype(u8),
    Tuple(u8, bool),
    Struct { x: u8 },
}

/// Each type tag, and the types written as other types
// schema[verify key.input.postcard]
// schema[verify key.input.bytes]
// schema[verify key.input.map]
// schema[verify key.input.tuple]
// schema[verify key.input.struct]
// schema[verify key.input.enum]
// schema[verify key.input.name]
#[test]
fn input_types() {
    fn key_of(ty: Type<Name, ()>) -> [u8; 8] {
        KeyInput::new(0x01, vec![In::Schema(ty)]).key()
    }
    let cases: Vec<([u8; 8], Type<Name, ()>)> = vec![
        (key::<bool>(), Type::Bool),
        (key::<i8>(), Type::I8),
        (key::<u8>(), Type::U8),
        (key::<i16>(), Type::I16),
        (key::<i32>(), Type::I32),
        (key::<i64>(), Type::I64),
        (key::<i128>(), Type::I128),
        (key::<u16>(), Type::U16),
        (key::<u32>(), Type::U32),
        (key::<u64>(), Type::U64),
        (key::<u128>(), Type::U128),
        (key::<f32>(), Type::F32),
        (key::<f64>(), Type::F64),
        (key::<char>(), Type::Char),
        (key::<str>(), Type::String(())),
        (key::<()>(), Type::Unit),
        (key::<Option<u8>>(), Type::option(Type::U8)),
        (key::<[u16]>(), Type::seq((), Type::U16)),
        (key::<MaxLenBytes<4>>(), Type::seq((), Type::U8)),
        (
            key::<(u8, bool)>(),
            Type::Aggregate(vec![el(None, Type::U8), el(None, Type::Bool)]),
        ),
        (
            key::<BTreeMap<u8, bool>>(),
            Type::seq(
                (),
                Type::Aggregate(vec![el(None, Type::U8), el(None, Type::Bool)]),
            ),
        ),
        (key::<OwnedDataModelType>(), Type::Schema),
        (key::<UnitStruct>(), Type::Unit),
        (key::<Newtype>(), Type::U8),
        (
            key::<TupleStruct>(),
            Type::Aggregate(vec![el(None, Type::U8), el(None, Type::Bool)]),
        ),
        (
            key::<Variants>(),
            Type::TaggedUnion(vec![
                el(Some("Unit"), Type::Unit),
                el(Some("Newtype"), Type::U8),
                el(
                    Some("Tuple"),
                    Type::Aggregate(vec![el(None, Type::U8), el(None, Type::Bool)]),
                ),
                el(
                    Some("Struct"),
                    Type::Aggregate(vec![el(Some("x"), Type::U8)]),
                ),
            ]),
        ),
    ];
    for (key, ty) in cases {
        let msg = format!("{ty:?}");
        assert_eq!(key, key_of(ty), "{msg}");
    }
}

/// With the names option disabled, no names are written for tuples, structs,
/// maps or enum variants
// schema[verify key.input.name]
#[test]
fn input_names_disabled() {
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Named {
        x: u8,
    }

    let key_of = |ty| KeyInput::<(), ()>::new(0x00, vec![In::Schema(ty)]).key();
    let pair = || Type::Aggregate(vec![el((), Type::U8), el((), Type::Bool)]);
    let cases: Vec<([u8; 8], Type<(), ()>)> = vec![
        (key_with::<(u8, bool)>(&NO_NAMES), pair()),
        (key_with::<TupleStruct>(&NO_NAMES), pair()),
        (
            key_with::<Named>(&NO_NAMES),
            Type::Aggregate(vec![el((), Type::U8)]),
        ),
        (
            key_with::<BTreeMap<u8, bool>>(&NO_NAMES),
            Type::seq((), pair()),
        ),
        (
            key_with::<Variants>(&NO_NAMES),
            Type::TaggedUnion(vec![
                el((), Type::Unit),
                el((), Type::U8),
                el((), pair()),
                el((), Type::Aggregate(vec![el((), Type::U8)])),
            ]),
        ),
    ];
    for (key, ty) in cases {
        let msg = format!("{ty:?}");
        assert_eq!(key, key_of(ty), "{msg}");
    }
}

/// Counts, lengths and maximum lengths of 128 or more, which postcard writes
/// as varints of more than one byte
// schema[verify key.input.postcard]
// schema[verify key.input.max-len]
#[test]
fn input_long_varints() {
    let long: &'static str = "x".repeat(200).leak();
    let opt = NO_NAMES.with_max_len(true);
    let key_of = |items| KeyInput::<(), Option<u64>>::new(0x02, items).key();

    assert_eq!(
        hasher(&opt, &[Item::String(long)]),
        key_of(vec![In::String(long)])
    );
    assert_eq!(
        key_with::<MaxLenString<300>>(&opt),
        key_of(vec![In::Schema(Type::String(Some(300)))])
    );
    assert_eq!(
        key_with::<[u8; 200]>(&opt),
        key_of(vec![In::Schema(Type::Aggregate(
            (0..200).map(|_| el((), Type::U8)).collect()
        ))])
    );
    let items: Vec<Item<'_>> = (0..130).map(|_| Item::String("a")).collect();
    assert_eq!(
        hasher(&opt, &items),
        key_of((0..130).map(|_| In::String("a")).collect())
    );
}

/// Maximum lengths, with the maximum lengths option enabled and the names
/// option disabled
// schema[verify key.input.max-len]
// schema[verify key.input.bytes]
// schema[verify key.input.map]
#[test]
fn input_max_lengths() {
    const SEQ: DataModelType = DataModelType::Seq {
        element: &DataModelType::U8,
        max_len: Some(3),
    };
    const MAP: DataModelType = DataModelType::Map {
        key: &DataModelType::U8,
        val: &DataModelType::Bool,
        max_len: Some(4),
    };
    let opt = NO_NAMES.with_max_len(true);
    let key_of = |ty| KeyInput::<(), Option<u64>>::new(0x02, vec![In::Schema(ty)]).key();
    let schema = |s: &DataModelType| hasher(&opt, &[Item::Schema(s)]);

    assert_eq!(
        key_with::<MaxLenString<8>>(&opt),
        key_of(Type::String(Some(8)))
    );
    assert_eq!(
        key_with::<MaxLenBytes<4>>(&opt),
        key_of(Type::seq(Some(4), Type::U8))
    );
    assert_eq!(schema(&SEQ), key_of(Type::seq(Some(3), Type::U8)));
    assert_eq!(
        schema(&MAP),
        key_of(Type::seq(
            Some(4),
            Type::Aggregate(vec![el((), Type::U8), el((), Type::Bool)]),
        ))
    );
}

/// A key with no items
// schema[verify key.items]
#[test]
fn input_no_items() {
    assert_eq!(
        hasher(&Options::DEFAULT, &[]),
        KeyInput::<Name, ()>::new(0x01, vec![]).key()
    );
}

#[derive(Schema)]
#[postcard(crate = crate)]
struct A {
    a: u8,
}
#[derive(Schema)]
#[postcard(crate = crate)]
struct B {
    b: u8,
}

/// Schemas with the same postcard type have the same key, and with the names
/// option disabled, so do schemas that differ only in their names.
#[test]
fn same_postcard_type_same_key() {
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Meters(u32);
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Pair(u8, i16);
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Empty;
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Named {
        a: u8,
        b: i16,
    }
    #[derive(Schema)]
    #[postcard(crate = crate)]
    enum E1 {
        A(Meters),
        B(u8, i16),
    }
    #[derive(Schema)]
    #[postcard(crate = crate)]
    enum E2 {
        A(u32),
        B((u8, i16)),
    }
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct OtherName {
        a: u8,
        b: i16,
    }

    assert_eq!(key::<Meters>(), key::<u32>());
    assert_eq!(key::<Pair>(), key::<(u8, i16)>());
    assert_eq!(key::<Empty>(), key::<()>());
    assert_eq!(key::<E1>(), key::<E2>());
    assert_eq!(key::<Vec<u8>>(), key::<MaxLenBytes<16>>());
    assert_eq!(key::<BTreeMap<u8, u16>>(), key::<Vec<(u8, u16)>>());
    // type names are never part of the key (`E1` and `E2` above also differ)
    assert_eq!(key::<Named>(), key::<OtherName>());
    // without names, a struct matches a tuple, and renamed fields match
    assert_eq!(key_with::<Named>(&NO_NAMES), key_with::<Pair>(&NO_NAMES));
    assert_eq!(key_with::<A>(&NO_NAMES), key_with::<B>(&NO_NAMES));
}

/// Schemas with different postcard types, or different names with the names
/// option enabled, have different keys, even where they are wire compatible for
/// some values (see the wire specification).
#[test]
fn different_postcard_type_different_key() {
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Pair(u8, i16);
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Named {
        a: u8,
        b: i16,
    }
    #[derive(Schema)]
    #[postcard(crate = crate)]
    enum MyOption {
        None,
        Some(u8),
    }
    #[derive(Schema)]
    #[postcard(crate = crate)]
    enum Ab {
        A,
        B,
    }
    #[derive(Schema)]
    #[postcard(crate = crate)]
    enum Ba {
        B,
        A,
    }

    assert_ne!(key::<A>(), key::<B>());
    assert_ne!(key::<Named>(), key::<Pair>());
    assert_ne!(key::<Ab>(), key::<Ba>());
    assert_ne!(key::<u16>(), key::<u32>());
    assert_ne!(key::<Option<u8>>(), key::<MyOption>());
    assert_ne!(key::<(u8,)>(), key::<u8>());
    assert_ne!(key::<((u8, u8), u8)>(), key::<(u8, u8, u8)>());
    assert_ne!(key::<String>(), key::<Vec<u8>>());
    assert_ne!(key::<Vec<u8>>(), key::<[u8; 1]>());
}

/// Maximum lengths are only part of the key with [`Options::max_len`].
#[test]
fn max_len_option() {
    assert_eq!(key::<MaxLenString<8>>(), key::<String>());
    assert_ne!(
        key_with::<MaxLenString<8>>(&MAX_LEN),
        key_with::<String>(&MAX_LEN)
    );
    assert_ne!(
        key_with::<MaxLenString<8>>(&MAX_LEN),
        key_with::<MaxLenString<9>>(&MAX_LEN)
    );
}

/// The owned hasher gives the same key as the const one.
#[test]
fn owned_matches_const() {
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Foo {
        a: u32,
        b: MaxLenString<32>,
        c: Vec<u8>,
        d: MaxLenBytes<4>,
        e: BTreeMap<u8, String>,
    }
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Wrap(Foo);
    #[derive(Schema)]
    #[postcard(crate = crate)]
    struct Unit;
    #[derive(Schema)]
    #[postcard(crate = crate)]
    enum Bar {
        A,
        B(Foo),
        C(u8, i64),
        D { x: Option<()>, y: (bool, char) },
        E(Wrap, Unit),
        F(OwnedDataModelType),
    }

    let owned: OwnedDataModelType = Bar::SCHEMA.into();
    for opt in [
        Options::DEFAULT,
        NO_NAMES,
        MAX_LEN,
        NO_NAMES.with_max_len(true),
    ] {
        assert_eq!(
            hasher(&opt, &[Item::String("p"), Item::Schema(Bar::SCHEMA)]),
            owned::hasher(
                &opt,
                &[
                    owned::OwnedItem::String("p"),
                    owned::OwnedItem::Schema(&owned)
                ]
            ),
        );
    }
    assert_eq!(
        Key::new(
            &Options::DEFAULT,
            &[Item::String("p"), Item::Schema(Bar::SCHEMA)]
        ),
        Key::new_owned(
            &Options::DEFAULT,
            &[
                owned::OwnedItem::String("p"),
                owned::OwnedItem::Schema(&owned)
            ]
        )
    );
}
