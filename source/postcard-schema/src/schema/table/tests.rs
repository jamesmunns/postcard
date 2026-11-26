use super::{
    NameRef, OwnedSchemaTable, OwnedTableData, OwnedTableNode as O, OwnedTableVariant,
    RawOwnedSchemaTable, SchemaTable, SchemaTableRef, Span, TableData, TableField, TableNode as N,
    TableVariant, VERSION,
};
use crate::{schema::OwnedDataModelType, Schema};

/// A const-built table, received
fn received(t: SchemaTableRef<'_>) -> OwnedSchemaTable {
    RawOwnedSchemaTable::from(t).validate().unwrap()
}

/// Checks that a const-built table is valid once received.
fn check<
    const ROOTS: usize,
    const NODES: usize,
    const CHILDREN: usize,
    const VARIANTS: usize,
    const FIELDS: usize,
    const NAME_BYTES: usize,
>(
    t: &SchemaTable<ROOTS, NODES, CHILDREN, VARIANTS, FIELDS, NAME_BYTES>,
) {
    assert_eq!(t.version, VERSION);
    received(t.as_table_ref());
}

#[test]
fn primitive() {
    let t = schema_table!(u8);
    check(&t);
    assert_eq!(t.nodes, [N::U8]);
    assert_eq!(t.roots, [0]);
}

#[derive(Schema)]
#[postcard(crate = crate)]
#[allow(dead_code)]
struct Pair(u8, bool);

/// Equal nodes are written once, and so are equal lists, even of different
/// nodes. A tuple of copies of one node is an array.
// schema[verify table.dedup]
#[test]
fn dedups_nodes_and_lists() {
    let t = schema_table!(((u8, bool), (u8, bool)), Pair);
    check(&t);
    let pair = Span { start: 0, len: 2 };
    assert_eq!(
        t.nodes,
        [
            N::U8,
            N::Bool,
            // (u8, bool)
            N::Tuple(pair),
            // [(u8, bool); 2]]
            N::Array { element: 2, len: 2 },
            // Pair
            N::Struct {
                name: NameRef { offset: 0, len: 4 },
                data: TableData::Tuple(pair),
            },
        ]
    );
    assert_eq!(t.children, [0, 1]);
    assert_eq!(t.roots, [3, 4]);

    // Once received, each node holds its own list.
    let owned = received(t.as_table_ref());
    assert_eq!(owned.nodes()[2], O::Tuple(vec![0, 1]));
    assert_eq!(
        owned.nodes()[4],
        O::Struct {
            name: NameRef { offset: 0, len: 4 },
            data: OwnedTableData::Tuple(vec![0, 1]),
        }
    );
}

#[derive(Schema)]
#[postcard(crate = crate)]
#[allow(dead_code)]
struct Bools(bool, bool);

/// A list that starts inside the lists written so far, and runs past
/// their end, is appended after the overlap. It then has one span, so equal
/// nodes are still written once.
// schema[verify table.dedup]
#[test]
fn dedups_overlapping_lists() {
    let t = schema_table!((u8, bool), Bools, (Bools, Bools));
    check(&t);
    // `[1, 1]` starts at the `1` of `[0, 1]`.
    assert_eq!(t.children, [0, 1, 1]);
    let bools = N::Struct {
        name: NameRef { offset: 0, len: 5 },
        data: TableData::Tuple(Span { start: 1, len: 2 }),
    };
    assert_eq!(
        t.nodes,
        [
            N::U8,
            N::Bool,
            N::Tuple(Span { start: 0, len: 2 }),
            bools,
            N::Array { element: 3, len: 2 },
        ]
    );
    assert_eq!(t.roots, [2, 3, 4]);
    let roots: Vec<OwnedDataModelType> = vec![
        <(u8, bool)>::SCHEMA.into(),
        Bools::SCHEMA.into(),
        <(Bools, Bools)>::SCHEMA.into(),
    ];
    assert_eq!(
        OwnedSchemaTable::new(&roots).unwrap(),
        received(t.as_table_ref())
    );
}

/// Arrays, and arrays of arrays, are one node each.
// schema[verify table.array]
#[test]
fn arrays() {
    let t = schema_table!([u8; 32], [[u8; 4]; 8], [u8; 1], [u8; 0]);
    check(&t);
    assert_eq!(
        t.nodes,
        [
            N::U8,
            // [u8; 32]
            N::Array {
                element: 0,
                len: 32
            },
            // [[u8; 4]; 8]
            N::Array { element: 0, len: 4 },
            N::Array { element: 2, len: 8 },
            // too short for arrays
            N::Tuple(Span { start: 0, len: 1 }),
            N::Tuple(Span { start: 0, len: 0 }),
        ]
    );
    assert_eq!(t.children, [0]);
    let decoded = received(t.as_table_ref()).decode().unwrap();
    assert_eq!(decoded[1], <[[u8; 4]; 8]>::SCHEMA.into());
    assert_eq!(decoded[3], <[u8; 0]>::SCHEMA.into());
}

#[derive(Schema)]
#[postcard(crate = crate)]
#[allow(dead_code)]
struct Point {
    xy: u8,
    y: u8,
}

#[derive(Schema)]
#[postcard(crate = crate)]
#[allow(dead_code)]
enum Shape {
    Dot(Point),
    Line(Point, Point),
    Empty,
}

/// Variants and fields aren't nodes: only types are.
// schema[verify table.dedup]
#[test]
fn names_are_interned() {
    let t = schema_table!(Shape);
    check(&t);
    // "y" is found inside "xy", and "Point" is written once.
    assert_eq!(&t.names, b"ShapeDotPointxyLineEmpty");
    let xy = NameRef { offset: 13, len: 2 };
    let y = NameRef { offset: 14, len: 1 };
    assert_eq!(t.as_table_ref().name(xy), "xy");
    // `Point`'s fields
    assert_eq!(
        t.fields,
        [
            TableField { name: xy, ty: 0 },
            TableField { name: y, ty: 0 },
        ]
    );
    assert_eq!(
        t.nodes,
        [
            N::U8,
            // `Point`
            N::Struct {
                name: NameRef { offset: 8, len: 5 },
                data: TableData::Struct(Span { start: 0, len: 2 }),
            },
            // `Shape`
            N::Enum {
                name: NameRef { offset: 0, len: 5 },
                variants: Span { start: 0, len: 3 },
            },
        ]
    );
    assert_eq!(
        t.variants,
        [
            // Dot(Point)
            TableVariant {
                name: NameRef { offset: 5, len: 3 },
                data: TableData::Newtype(1),
            },
            // Line(Point, Point)
            TableVariant {
                name: NameRef { offset: 15, len: 4 },
                data: TableData::Tuple(Span { start: 0, len: 2 }),
            },
            // Empty
            TableVariant {
                name: NameRef { offset: 19, len: 5 },
                data: TableData::Unit,
            },
        ]
    );
    assert_eq!(t.children, [1, 1]);
    assert_eq!(t.roots, [2]);
}

#[test]
fn roots_share_nodes() {
    let t = schema_table!(u8, Option<u8>, (Option<u8>,));
    check(&t);
    assert_eq!(t.roots, [0, 1, 2]);
    assert_eq!(t.nodes[2], N::Tuple(Span { start: 0, len: 1 }));
    assert_eq!(t.children, [1]);
}

#[test]
fn max_len() {
    let t = schema_table!(crate::max_len::MaxLenStr<'static, 16>);
    assert_eq!(t.nodes, [N::String { max_len: Some(16) }]);
}

/// Lists nested inside lists, none of them shared, fit in the const builder's
/// scratch space.
#[test]
fn nested_lists() {
    type Deep = ((u8, i8), (u16, (i16, u32)), (i32, (u64, (i64, bool))));
    let t = schema_table!(Deep, Shape);
    check(&t);
    let owned = received(t.as_table_ref());
    assert_eq!(
        owned.decode(),
        Ok(vec![Deep::SCHEMA.into(), Shape::SCHEMA.into()])
    );
}

/// A table can be built in a `static`, with its type written out.
#[test]
fn in_a_static() {
    static T: SchemaTable<1, 2, 0, 0, 0, 3> = schema_table!(New);
    #[derive(Schema)]
    #[postcard(crate = crate)]
    #[allow(dead_code)]
    struct New(u8);
    check(&T);
    assert_eq!(
        T.nodes[1],
        N::Struct {
            name: NameRef { offset: 0, len: 3 },
            data: TableData::Newtype(0)
        }
    );
}

/// The encoding of a table, field by field.
// schema[verify table.encoding]
#[test]
fn encoding() {
    let t = schema_table!(u8);
    let bytes = postcard::to_vec(&t).unwrap();
    // version, roots, nodes (`U8` is variant 2), names
    assert_eq!(bytes, [1, 1, 0, 1, 2, 0]);
    assert_eq!(postcard::to_vec(&t.as_table_ref()).unwrap(), bytes);
}

/// A sequence of at most four `u8`s
struct ShortSeq;

impl Schema for ShortSeq {
    const SCHEMA: &'static crate::schema::DataModelType = &crate::schema::DataModelType::Seq {
        element: u8::SCHEMA,
        max_len: Some(4),
    };
}

/// A map of at most four `u8`s to `u8`s
struct ShortMap;

impl Schema for ShortMap {
    const SCHEMA: &'static crate::schema::DataModelType = &crate::schema::DataModelType::Map {
        key: u8::SCHEMA,
        val: u8::SCHEMA,
        max_len: Some(4),
    };
}

/// Every kind of type, and maximum lengths
#[derive(Schema)]
#[postcard(crate = crate)]
#[allow(dead_code)]
struct Kinds {
    a: bool,
    b: i8,
    c: u8,
    d: i16,
    e: i32,
    f: i64,
    g: i128,
    h: u16,
    i: u32,
    j: u64,
    k: u128,
    l: f32,
    m: f64,
    n: char,
    o: String,
    p: crate::max_len::MaxLenString<8>,
    q: crate::max_len::MaxLenBytes<8>,
    r: Option<u8>,
    s: (),
    t: Vec<u8>,
    u: ShortSeq,
    v: (u8, bool),
    w: std::collections::BTreeMap<u8, u8>,
    x: ShortMap,
    y: Shape,
    z: [u16; 3],
}

/// Pairs of types that differ in one part, which must not be written as one
/// node
#[allow(dead_code)]
mod near {
    pub mod one {
        use crate::Schema;

        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub struct Ab;
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub struct N(pub u8);
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub struct T(pub u8, pub u8);
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub struct S {
            pub a: u8,
        }
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub enum E {
            X(u8),
        }
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub enum V {
            X,
        }
    }

    pub mod two {
        use crate::Schema;

        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub struct A;
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub struct N(pub u16);
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub struct T(pub u8, pub u16);
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub struct S {
            pub a: u16,
        }
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub enum E {
            X(u16),
        }
        #[derive(Schema)]
        #[postcard(crate = crate)]
        pub enum W {
            X,
        }
    }
}

/// The runtime builder gives the same table as the const one, and an owned
/// table round-trips through postcard.
#[test]
fn owned_matches_const() {
    #[track_caller]
    fn same<
        const ROOTS: usize,
        const NODES: usize,
        const CHILDREN: usize,
        const VARIANTS: usize,
        const FIELDS: usize,
        const NAME_BYTES: usize,
    >(
        t: &SchemaTable<ROOTS, NODES, CHILDREN, VARIANTS, FIELDS, NAME_BYTES>,
        roots: &[OwnedDataModelType],
    ) {
        let owned = OwnedSchemaTable::new(roots).unwrap();
        assert_eq!(received(t.as_table_ref()), owned);
        let bytes = postcard::to_vec(&owned).unwrap();
        assert_eq!(bytes, postcard::to_vec(t).unwrap());
        assert_eq!(
            postcard::from_bytes::<OwnedSchemaTable>(&bytes).unwrap(),
            owned
        );
    }

    same(&schema_table!(u8), &[u8::SCHEMA.into()]);
    same(
        &schema_table!(((u8, bool), (u8, bool)), Pair),
        &[
            <((u8, bool), (u8, bool))>::SCHEMA.into(),
            Pair::SCHEMA.into(),
        ],
    );
    same(&schema_table!(Shape), &[Shape::SCHEMA.into()]);
    same(
        &schema_table!(u8, Option<u8>, (Option<u8>,)),
        &[
            u8::SCHEMA.into(),
            <Option<u8>>::SCHEMA.into(),
            <(Option<u8>,)>::SCHEMA.into(),
        ],
    );
    same(
        &schema_table!(Result<Vec<Shape>, std::collections::BTreeMap<String, [u8; 4]>>),
        &[<Result<Vec<Shape>, std::collections::BTreeMap<String, [u8; 4]>>>::SCHEMA.into()],
    );
    same(&schema_table!(Kinds), &[Kinds::SCHEMA.into()]);
}

/// Decoding a table gives back the schemas it was built from.
// schema[verify table.meaning]
#[test]
fn decode_round_trips() {
    type Big = Result<Vec<Shape>, std::collections::BTreeMap<String, [u8; 4]>>;
    let t = schema_table!(Shape, Pair, Big, u8, Kinds);
    let expected: Vec<OwnedDataModelType> = vec![
        Shape::SCHEMA.into(),
        Pair::SCHEMA.into(),
        Big::SCHEMA.into(),
        u8::SCHEMA.into(),
        Kinds::SCHEMA.into(),
    ];
    let owned = received(t.as_table_ref());
    assert_eq!(owned.decode(), Ok(expected));
}

/// Similar, not the same
// schema[verify table.meaning]
#[test]
fn near_duplicates_are_kept_apart() {
    use near::{one, two};
    macro_rules! both {
        ($($ty:ty),+) => {{
            let t = schema_table!($($ty),+);
            check(&t);
            let expected: Vec<OwnedDataModelType> = vec![$(<$ty>::SCHEMA.into()),+];
            let owned = received(t.as_table_ref());
            assert_eq!(owned.decode(), Ok(expected.clone()));
            assert_eq!(OwnedSchemaTable::new(&expected), Ok(owned));
        }};
    }
    both!(
        // inner type
        Option<u8>,
        Option<u16>,
        // maximum lengths
        Vec<u8>,
        ShortSeq,
        std::collections::BTreeMap<u8, u8>,
        ShortMap,
        crate::max_len::MaxLenString<4>,
        crate::max_len::MaxLenString<8>,
        // array length
        [u8; 2],
        [u8; 3],
        // name inside a name
        one::Ab,
        two::A,
        // struct data
        one::N,
        two::N,
        one::T,
        two::T,
        // field type
        one::S,
        two::S,
        // variant data
        one::E,
        two::E,
        // enum name
        one::V,
        two::W
    );
}

mod malformed {
    use super::super::{DecodeConfig, TableError as E};
    use super::*;

    /// The default config, with a limit of `n` nodes
    fn max_nodes(n: usize) -> DecodeConfig {
        DecodeConfig {
            max_nodes: n,
            ..DecodeConfig::default()
        }
    }

    const NAME: NameRef = NameRef { offset: 0, len: 1 };

    /// Validating, then decoding, a table built by hand
    trait Check {
        fn valid(&self) -> Result<(), E>;
        fn decode(&self) -> Result<Vec<OwnedDataModelType>, E>;
        fn decode_with_config(&self, c: &DecodeConfig) -> Result<Vec<OwnedDataModelType>, E>;
    }

    impl Check for RawOwnedSchemaTable {
        fn valid(&self) -> Result<(), E> {
            self.clone().validate().map(drop)
        }

        fn decode(&self) -> Result<Vec<OwnedDataModelType>, E> {
            self.clone().validate()?.decode()
        }

        fn decode_with_config(&self, c: &DecodeConfig) -> Result<Vec<OwnedDataModelType>, E> {
            self.clone().validate()?.decode_with_config(c)
        }
    }

    fn table(nodes: Vec<O>, names: &str) -> RawOwnedSchemaTable {
        RawOwnedSchemaTable {
            version: VERSION,
            roots: vec![0],
            nodes,
            names: names.into(),
        }
    }

    // schema[verify table.version]
    #[test]
    fn version() {
        let mut t = table(vec![O::U8], "");
        t.version = 2;
        assert_eq!(t.valid(), Err(E::UnsupportedVersion(2)));
        t.version = 0;
        assert_eq!(t.valid(), Err(E::UnsupportedVersion(0)));
    }

    // schema[verify table.root]
    #[test]
    fn roots() {
        let mut t = table(vec![O::U8], "");
        t.roots = vec![1];
        assert_eq!(t.valid(), Err(E::BadRoot(1)));
    }

    /// A node must only refer to nodes before it, so a table can't have a
    /// cycle, even in a node no root reaches.
    // schema[verify table.child.order]
    // schema[verify table.reject]
    #[test]
    fn children_come_first() {
        // refers to itself
        let t = table(vec![O::Option(0)], "");
        assert_eq!(t.valid(), Err(E::BadChild { node: 0, child: 0 }));
        // unreachable cycle
        let t = table(vec![O::U8, O::Option(2), O::Option(1)], "");
        assert_eq!(t.valid(), Err(E::BadChild { node: 1, child: 2 }));
        // array of itself
        let t = table(vec![O::Array { element: 0, len: 2 }], "");
        assert_eq!(t.valid(), Err(E::BadChild { node: 0, child: 0 }));
    }

    /// Every index inside a node's lists is checked: tuple elements, and the
    /// data of variants and the types of fields.
    // schema[verify table.child.order]
    #[test]
    fn indices_in_lists() {
        let bad = |node| E::BadChild { node, child: 7 };
        // bad in tuple element
        let t = table(vec![O::U8, O::Tuple(vec![0, 7])], "");
        assert_eq!(t.valid(), Err(bad(1)));
        // bad in seq element
        let seq = O::Seq {
            element: 7,
            max_len: None,
        };
        assert_eq!(table(vec![O::U8, seq], "").valid(), Err(bad(1)));
        let map = |key, val| O::Map {
            key,
            val,
            max_len: None,
        };
        // bad in map key and value
        assert_eq!(table(vec![O::U8, map(7, 0)], "").valid(), Err(bad(1)));
        assert_eq!(table(vec![O::U8, map(0, 7)], "").valid(), Err(bad(1)));
        let variant = |data| OwnedTableVariant { name: NAME, data };
        let enum_of = |data| O::Enum {
            name: NAME,
            variants: vec![variant(OwnedTableData::Unit), variant(data)],
        };
        // bad in newtype variant
        let t = table(vec![O::U8, enum_of(OwnedTableData::Newtype(7))], "a");
        assert_eq!(t.valid(), Err(bad(1)));
        // bad in tuple variant
        let t = table(vec![O::U8, enum_of(OwnedTableData::Tuple(vec![0, 7]))], "a");
        assert_eq!(t.valid(), Err(bad(1)));
        // bad in struct variant field
        let field = TableField { name: NAME, ty: 7 };
        let t = table(
            vec![O::U8, enum_of(OwnedTableData::Struct(vec![field]))],
            "a",
        );
        assert_eq!(t.valid(), Err(bad(1)));
        // bad in struct field
        let t = table(
            vec![
                O::U8,
                O::Struct {
                    name: NAME,
                    data: OwnedTableData::Struct(vec![field]),
                },
            ],
            "a",
        );
        assert_eq!(t.valid(), Err(bad(1)));
    }

    /// Every name is checked: of structs, enums, variants and fields.
    // schema[verify table.name]
    #[test]
    fn names() {
        let unit = |name| O::Struct {
            name,
            data: OwnedTableData::Unit,
        };
        // past the end
        let t = table(vec![O::U8, unit(NAME)], "");
        assert_eq!(
            t.valid(),
            Err(E::BadName {
                node: 1,
                name: NAME
            })
        );
        // Half of a two-byte character
        let t = table(vec![O::U8, unit(NAME)], "é");
        assert_eq!(
            t.valid(),
            Err(E::BadName {
                node: 1,
                name: NAME
            })
        );
        // The other half
        let half = NameRef { offset: 1, len: 1 };
        let t = table(vec![O::U8, unit(half)], "é");
        assert_eq!(
            t.valid(),
            Err(E::BadName {
                node: 1,
                name: half
            })
        );
        // An enum's own name
        let t = table(
            vec![
                O::U8,
                O::Enum {
                    name: NAME,
                    variants: vec![],
                },
            ],
            "",
        );
        assert_eq!(
            t.valid(),
            Err(E::BadName {
                node: 1,
                name: NAME
            })
        );
        // a variant's name, overflowing
        let far = NameRef {
            offset: u32::MAX,
            len: u32::MAX,
        };
        let t = table(
            vec![
                O::U8,
                O::Enum {
                    name: NAME,
                    variants: vec![OwnedTableVariant {
                        name: far,
                        data: OwnedTableData::Unit,
                    }],
                },
            ],
            "a",
        );
        assert_eq!(t.valid(), Err(E::BadName { node: 1, name: far }));
        // a field's name
        let t = table(
            vec![
                O::U8,
                O::Struct {
                    name: NAME,
                    data: OwnedTableData::Struct(vec![TableField { name: far, ty: 0 }]),
                },
            ],
            "a",
        );
        assert_eq!(t.valid(), Err(E::BadName { node: 1, name: far }));
    }

    /// Repeated nodes and names, overlapping names, and nodes no root refers to
    /// are all valid.
    // schema[verify table.accept]
    #[test]
    fn accepts_any_valid_table() {
        use crate::schema::OwnedDataModelType as T;
        let nodes = vec![
            // a repeated node
            O::U8,
            O::U8,
            O::Bool,
            O::Tuple(vec![0, 1]),
            O::Tuple(vec![1, 2]),
            // unreachable, named "ab"
            O::Struct {
                name: NameRef { offset: 0, len: 2 },
                data: OwnedTableData::Unit,
            },
            // named "b", inside "ab"
            O::Struct {
                name: NameRef { offset: 1, len: 1 },
                data: OwnedTableData::Unit,
            },
        ];
        let mut t = table(nodes, "ab");
        // a repeated root
        t.roots = vec![3, 4, 6, 6];
        let unit = || T::Struct {
            name: "b".into(),
            data: crate::schema::OwnedData::Unit,
        };
        assert_eq!(
            t.decode_with_config(&max_nodes(100)),
            Ok(vec![
                T::Tuple([T::U8, T::U8].into()),
                T::Tuple([T::U8, T::Bool].into()),
                unit(),
                unit(),
            ])
        );
    }

    /// A maximum length, or an array's length, must fit in a `usize`.
    /// The limit is a parameter, so each length is checked on any target.
    // schema[verify table.max-len]
    #[test]
    fn max_len() {
        let nodes = [
            O::String { max_len: Some(11) },
            O::ByteArray { max_len: Some(11) },
            O::Seq {
                element: 0,
                max_len: Some(11),
            },
            O::Map {
                key: 0,
                val: 0,
                max_len: Some(11),
            },
            O::Array {
                element: 0,
                len: 11,
            },
        ];
        for node in nodes {
            let t = table(vec![O::U8, node], "");
            assert_eq!(t.check(11), Ok(()));
            assert_eq!(t.check(10), Err(E::LenTooLarge { node: 1 }));
        }
        let t = table(
            vec![O::String {
                max_len: Some(u64::MAX),
            }],
            "",
        );
        if usize::BITS < 64 {
            assert_eq!(t.valid(), Err(E::LenTooLarge { node: 0 }));
        } else {
            assert_eq!(t.valid(), Ok(()));
        }
    }

    /// Arrays are checked against the budget before any copies are made.
    // schema[verify table.budget]
    #[test]
    fn huge_arrays() {
        // 2^32 - 1 `U8`s
        let mut t = table(
            vec![
                O::U8,
                O::Array {
                    element: 0,
                    len: u32::MAX as u64,
                },
            ],
            "",
        );
        t.roots = vec![1];
        assert_eq!(t.valid(), Ok(()));
        assert_eq!(
            t.decode_with_config(&max_nodes(1_000_000)),
            Err(E::LimitExceeded)
        );
        // An array of 1000 arrays of 1000 `U8`s, and the arrays
        let mut t = table(
            vec![
                O::U8,
                O::Array {
                    element: 0,
                    len: 1000,
                },
                O::Array {
                    element: 1,
                    len: 1000,
                },
            ],
            "",
        );
        t.roots = vec![2];
        assert_eq!(
            t.decode_with_config(&max_nodes(1_000_000)),
            Err(E::LimitExceeded)
        );
        assert_eq!(
            t.decode_with_config(&max_nodes(1_001_001)).map(|v| v.len()),
            Ok(1)
        );
        assert_eq!(
            t.decode_with_config(&max_nodes(1_001_000)),
            Err(E::LimitExceeded)
        );
    }

    /// 64 nodes, each a pair of the one before, describe 2^64 `U8`s.
    // schema[verify table.budget]
    #[test]
    fn exponential() {
        let mut nodes = vec![O::U8];
        for i in 0..64u32 {
            nodes.push(O::Tuple(vec![i, i]));
        }
        let mut t = table(nodes, "");
        t.roots = vec![64];
        assert_eq!(t.valid(), Ok(()));
        assert_eq!(
            t.decode_with_config(&max_nodes(100_000)),
            Err(E::LimitExceeded)
        );
        t.roots = vec![3];
        // 8 `U8`s and 7 tuples
        assert_eq!(t.decode_with_config(&max_nodes(15)).map(|v| v.len()), Ok(1));
        assert_eq!(t.decode_with_config(&max_nodes(14)), Err(E::LimitExceeded));
    }

    /// Each use of a name counts towards the name byte limit, including
    /// in the copies of an array.
    // schema[verify table.budget]
    #[test]
    fn name_bytes() {
        let long = "x".repeat(1000);
        let name = NameRef {
            offset: 0,
            len: 1000,
        };
        let unit = O::Struct {
            name,
            data: OwnedTableData::Unit,
        };
        // One 1000-byte name, in an array of 64 arrays of 64 structs
        let mut t = table(
            vec![
                unit,
                O::Array {
                    element: 0,
                    len: 64,
                },
                O::Array {
                    element: 1,
                    len: 64,
                },
            ],
            &long,
        );
        t.roots = vec![2];
        assert_eq!(t.valid(), Ok(()));
        let mut config = DecodeConfig {
            max_name_bytes: 64 * 64 * 1000 - 1,
            ..DecodeConfig::default()
        };
        assert_eq!(t.decode_with_config(&config), Err(E::LimitExceeded));
        config.max_name_bytes += 1;
        assert_eq!(t.decode_with_config(&config).map(|v| v.len()), Ok(1));
    }

    /// A chain of nested types is limited by depth, whatever the other limits,
    /// so decoding can't overflow the stack.
    // schema[verify table.budget]
    #[test]
    fn depth() {
        let mut nodes_list = vec![O::U8];
        for i in 0..10_000u32 {
            nodes_list.push(O::Option(i));
        }
        let mut t = table(nodes_list, "");
        t.roots = vec![10_000];
        assert_eq!(t.valid(), Ok(()));
        assert_eq!(
            t.decode_with_config(&max_nodes(usize::MAX)),
            Err(E::LimitExceeded)
        );
        // 128 nested types are allowed by default, and 129 aren't.
        t.roots = vec![127];
        assert_eq!(t.decode().map(|v| v.len()), Ok(1));
        t.roots = vec![128];
        assert_eq!(t.decode(), Err(E::LimitExceeded));
    }

    /// Deserializing an `OwnedSchemaTable` validates it, and a raw table can be
    /// read without validating.
    // schema[verify table.reject]
    #[test]
    fn deserializing_validates() {
        let bad = table(vec![O::Option(0)], "");
        let bytes = postcard::to_vec(&bad).unwrap();
        assert!(postcard::from_bytes::<OwnedSchemaTable>(&bytes).is_err());
        let raw: RawOwnedSchemaTable = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(raw, bad);
        assert_eq!(
            OwnedSchemaTable::try_from(raw),
            Err(E::BadChild { node: 0, child: 0 })
        );

        let good = table(vec![O::U8], "");
        let bytes = postcard::to_vec(&good).unwrap();
        let t: OwnedSchemaTable = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(postcard::to_vec(&t).unwrap(), bytes);
        assert_eq!(t.into_raw(), good);
    }

    /// An `Array` of no copies is an empty tuple, and of one copy is a tuple of
    /// one.
    // schema[verify table.array]
    #[test]
    fn short_arrays() {
        use crate::schema::OwnedDataModelType as T;
        let mut t = table(
            vec![
                O::U8,
                O::Array { element: 0, len: 0 },
                O::Array { element: 0, len: 1 },
            ],
            "",
        );
        t.roots = vec![1, 2];
        assert_eq!(
            t.decode(),
            Ok(vec![T::Tuple([].into()), T::Tuple([T::U8].into())])
        );
    }

    /// The limits are for all of the roots together.
    // schema[verify table.budget]
    #[test]
    fn budget_is_shared_by_roots() {
        let mut nodes = vec![O::U8];
        for i in 0..3u32 {
            nodes.push(O::Tuple(vec![i, i]));
        }
        let mut t = table(nodes, "");
        // 15 nodes each
        t.roots = vec![3, 3];
        assert_eq!(t.decode_with_config(&max_nodes(29)), Err(E::LimitExceeded));
        assert_eq!(t.decode_with_config(&max_nodes(30)).map(|v| v.len()), Ok(2));
    }

    /// Depth is the longest path from a root, not the number of nested types:
    /// a tuple of 200 `Option<U8>`s is three deep.
    // schema[verify table.budget]
    #[test]
    fn depth_is_per_path() {
        let mut t = table(vec![O::U8, O::Option(0), O::Tuple(vec![1; 200])], "");
        t.roots = vec![2];
        let depth = |max_depth| DecodeConfig {
            max_depth,
            ..DecodeConfig::default()
        };
        assert_eq!(t.decode_with_config(&depth(3)).map(|v| v.len()), Ok(1));
        assert_eq!(t.decode_with_config(&depth(2)), Err(E::LimitExceeded));
    }

    /// Sizes too large to count are rejected, whatever the limits.
    // schema[verify table.budget]
    #[test]
    fn sizes_that_overflow() {
        let unlimited = DecodeConfig {
            max_nodes: usize::MAX,
            max_name_bytes: usize::MAX,
            max_depth: usize::MAX,
        };
        // 2^64 `U8`s, and more than 2^64 nodes
        let mut nodes = vec![O::U8];
        for i in 0..64u32 {
            nodes.push(O::Tuple(vec![i, i]));
        }
        let mut t = table(nodes, "");
        t.roots = vec![64];
        assert_eq!(t.decode_with_config(&unlimited), Err(E::LimitExceeded));
        // Copies of a pair, which overflow when multiplied
        let mut t = table(
            vec![
                O::U8,
                O::Tuple(vec![0, 0]),
                O::Array {
                    element: 1,
                    len: usize::MAX as u64,
                },
            ],
            "",
        );
        t.roots = vec![2];
        assert_eq!(t.decode_with_config(&unlimited), Err(E::LimitExceeded));
    }

    /// Variants and fields count towards the budget, as well as types.
    // schema[verify table.budget]
    #[test]
    fn budget_counts_variants_and_fields() {
        let field = TableField { name: NAME, ty: 0 };
        let t = table(
            vec![
                O::U8,
                O::Enum {
                    name: NAME,
                    variants: vec![OwnedTableVariant {
                        name: NAME,
                        data: OwnedTableData::Struct(vec![field, field]),
                    }],
                },
            ],
            "a",
        );
        let mut t = t;
        t.roots = vec![1];
        // The enum, its variant, two fields, and their two `U8`s
        assert_eq!(t.decode_with_config(&max_nodes(6)).map(|v| v.len()), Ok(1));
        assert_eq!(t.decode_with_config(&max_nodes(5)), Err(E::LimitExceeded));
    }
}

/// The example in the Schema Tables chapter of the specification.
// schema[verify table.encoding]
#[test]
fn spec_example() {
    let bytes = postcard::to_vec(&schema_table!(Point)).unwrap();
    let mut expected = vec![
        0x01, 0x01, 0x01, 0x02, 0x02, 0x15, 0x00, 0x05, 0x03, 0x02, 0x05, 0x02, 0x00, 0x06, 0x01,
        0x00, 0x07,
    ];
    expected.extend(b"Pointxy");
    assert_eq!(bytes, expected);
}

/// The const encoder writes the same bytes as postcard.
// schema[verify table.encoding]
#[test]
fn const_encoding_matches_postcard() {
    type Big = Result<Vec<Shape>, std::collections::BTreeMap<String, [u8; 4]>>;
    static BYTES: &[u8] = &schema_table_bytes!(Shape, Pair, Big, u8);
    let table = schema_table!(Shape, Pair, Big, u8);
    assert_eq!(BYTES, postcard::to_vec(&table).unwrap());
    assert_eq!(table.encoded_len(), BYTES.len());
    let owned: OwnedSchemaTable = postcard::from_bytes(BYTES).unwrap();
    assert_eq!(owned, received(table.as_table_ref()));

    // Every node kind, and values that take several bytes as varints
    let span = Span { start: 5, len: 150 };
    let name = NameRef {
        offset: 128,
        len: 70_000,
    };
    let max = Some(u64::MAX);
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
        N::String { max_len: max },
        N::ByteArray { max_len: None },
        N::Option(u32::MAX),
        N::Unit,
        N::Seq {
            element: 200,
            max_len: Some(0),
        },
        N::Tuple(span),
        N::Map {
            key: 1,
            val: 1 << 14,
            max_len: max,
        },
        N::Struct {
            name,
            data: TableData::Unit,
        },
        N::Struct {
            name,
            data: TableData::Newtype(129),
        },
        N::Struct {
            name,
            data: TableData::Tuple(span),
        },
        N::Struct {
            name,
            data: TableData::Struct(Span { start: 0, len: 2 }),
        },
        N::Enum {
            name,
            variants: Span { start: 0, len: 3 },
        },
        N::Array {
            element: 300,
            len: u64::MAX,
        },
    ];
    let children: Vec<u32> = (0..200).map(|i| i * 1000).collect();
    let variants = [
        TableVariant {
            name,
            data: TableData::Unit,
        },
        TableVariant {
            name,
            data: TableData::Tuple(span),
        },
        TableVariant {
            name,
            data: TableData::Struct(Span { start: 1, len: 1 }),
        },
    ];
    let fields = [
        TableField { name, ty: 128 },
        TableField { name, ty: u32::MAX },
    ];
    let t = SchemaTableRef {
        version: 1,
        roots: &[0, 127, 128, u32::MAX],
        nodes: &nodes,
        children: &children,
        variants: &variants,
        fields: &fields,
        names: "schémas",
    };
    let expected = postcard::to_vec(&t).unwrap();
    assert_eq!(t.encoded_len(), expected.len());
    let mut buf = vec![0; expected.len()];
    assert_eq!(t.encode_into(&mut buf), Some(expected.len()));
    assert_eq!(buf, expected);
    let mut short = vec![0; expected.len() - 1];
    assert_eq!(t.encode_into(&mut short), None);
    // The owned form, which holds the lists in the nodes, encodes the same.
    assert_eq!(
        postcard::to_vec(&RawOwnedSchemaTable::from(t)).unwrap(),
        expected
    );
}

/// The runtime builder takes time linear in the size of the schemas: a struct
/// of 200,000 fields, with distinct names and types, builds in about a
/// second. Searching the nodes for each new one instead takes minutes.
#[test]
fn runtime_builder_is_linear() {
    use crate::schema::{OwnedData, OwnedNamedField};
    let fields: Vec<OwnedNamedField> = (0..200_000)
        .map(|i| OwnedNamedField {
            name: format!("field_{i}").into(),
            ty: OwnedDataModelType::Seq {
                element: Box::new(OwnedDataModelType::U8),
                max_len: Some(i),
            },
        })
        .collect();
    let ty = OwnedDataModelType::Struct {
        name: "Wide".into(),
        data: OwnedData::Struct(fields.into()),
    };
    let start = std::time::Instant::now();
    let config = super::DecodeConfig {
        max_nodes: 1_000_000,
        max_name_bytes: 4 << 20,
        ..super::DecodeConfig::default()
    };
    let t = OwnedSchemaTable::new_with_config(core::slice::from_ref(&ty), &config).unwrap();
    assert_eq!(t.decode_with_config(&config), Ok(vec![ty]));
    // Generous, so this only fails if building is far from linear
    assert!(start.elapsed() < std::time::Duration::from_secs(10));
}

/// Each distinct type is a root once, in the order first given, in both
/// builders.
// schema[verify table.dedup]
#[test]
fn roots_are_deduplicated() {
    let t = schema_table!(u8, Option<u8>, u8, Option<u8>, Pair, u8);
    check(&t);
    assert_eq!(t.roots, [0, 1, 3]);
    let roots: Vec<OwnedDataModelType> = vec![
        u8::SCHEMA.into(),
        <Option<u8>>::SCHEMA.into(),
        u8::SCHEMA.into(),
        Pair::SCHEMA.into(),
    ];
    let owned = OwnedSchemaTable::new(&roots).unwrap();
    assert_eq!(owned, received(t.as_table_ref()));
    assert_eq!(
        owned.decode(),
        Ok(vec![
            u8::SCHEMA.into(),
            <Option<u8>>::SCHEMA.into(),
            Pair::SCHEMA.into()
        ])
    );
}

/// The runtime builder has the same limits as decoding, counted the same way,
/// so a table it builds decodes with the same config.
// schema[verify table.budget]
#[test]
fn runtime_builder_limits() {
    use super::{DecodeConfig, TableError as E};
    let config = |max_nodes, max_name_bytes, max_depth| DecodeConfig {
        max_nodes,
        max_name_bytes,
        max_depth,
    };
    let build = |c: &DecodeConfig| {
        let t =
            OwnedSchemaTable::new_with_config(&[Point::SCHEMA.into(), Point::SCHEMA.into()], c)?;
        t.decode_with_config(c).map(|v| v.len())
    };
    // Two of: the struct, two fields and their two `U8`s, with 8 bytes of
    // names, two deep.
    assert_eq!(build(&config(10, 16, 2)), Ok(1));
    assert_eq!(build(&config(9, 16, 2)), Err(E::LimitExceeded));
    assert_eq!(build(&config(10, 15, 2)), Err(E::LimitExceeded));
    assert_eq!(build(&config(10, 16, 1)), Err(E::LimitExceeded));

    // Nested types are counted without recursion, so building stops at the
    // depth limit.
    let mut ty = OwnedDataModelType::U8;
    for _ in 0..127 {
        ty = OwnedDataModelType::Option(Box::new(ty));
    }
    assert!(OwnedSchemaTable::new(core::slice::from_ref(&ty)).is_ok());
    let ty = OwnedDataModelType::Option(Box::new(ty));
    assert_eq!(
        OwnedSchemaTable::new(core::slice::from_ref(&ty)),
        Err(E::LimitExceeded)
    );
}

/// Each node and data kind is written with the discriminant given in
/// the specification.
// schema[verify table.encoding]
#[test]
fn discriminants() {
    let none = NameRef { offset: 0, len: 0 };
    let empty = Span { start: 0, len: 0 };
    // The bytes of a table of one node, after the version, roots and count
    let node = |n: N| {
        let t = SchemaTableRef {
            version: 1,
            roots: &[],
            nodes: &[n],
            children: &[],
            variants: &[],
            fields: &[],
            names: "",
        };
        let bytes = postcard::to_vec(&t).unwrap();
        assert_eq!(bytes[..3], [1, 0, 1]);
        bytes[3..bytes.len() - 1].to_vec()
    };
    let nodes = [
        (N::Bool, 0x00),
        (N::I8, 0x01),
        (N::U8, 0x02),
        (N::I16, 0x03),
        (N::I32, 0x04),
        (N::I64, 0x05),
        (N::I128, 0x06),
        (N::U16, 0x07),
        (N::U32, 0x08),
        (N::U64, 0x09),
        (N::U128, 0x0A),
        (N::F32, 0x0B),
        (N::F64, 0x0C),
        (N::Char, 0x0D),
        (N::String { max_len: None }, 0x0E),
        (N::ByteArray { max_len: None }, 0x0F),
        (N::Option(0), 0x10),
        (N::Unit, 0x11),
        (
            N::Seq {
                element: 0,
                max_len: None,
            },
            0x12,
        ),
        (N::Tuple(empty), 0x13),
        (
            N::Map {
                key: 0,
                val: 0,
                max_len: None,
            },
            0x14,
        ),
        (
            N::Struct {
                name: none,
                data: TableData::Unit,
            },
            0x15,
        ),
        (
            N::Enum {
                name: none,
                variants: empty,
            },
            0x16,
        ),
        (N::Array { element: 0, len: 0 }, 0x17),
    ];
    for (n, d) in nodes {
        assert_eq!(node(n)[0], d, "{n:?}");
    }
    let data = [
        (TableData::Unit, 0x00),
        (TableData::Newtype(0), 0x01),
        (TableData::Tuple(empty), 0x02),
        (TableData::Struct(empty), 0x03),
    ];
    for (data, d) in data {
        // `Struct`, then an empty name
        assert_eq!(node(N::Struct { name: none, data })[..4], [0x15, 0, 0, d]);
    }
}
