use postcard_schema::{
    schema::{Data, DataModelType, NamedField, OwnedDataModelType, Variant},
    Schema,
};
use std::path::PathBuf;

#[allow(unused)]
#[derive(Schema)]
#[serde(rename = "Inner'")]
enum Inner {
    #[serde(rename = "⍺")]
    Alpha,
    Beta,
    Gamma,
    Delta(i32, i16),
    Epsilon {
        zeta: f32,
        eta: bool,
    },
}

#[allow(unused)]
#[derive(Schema)]
#[serde(rename = "Outer'")]
struct Outer<'a> {
    a: u32,
    #[serde(rename = "b'")]
    b: u64,
    c: u8,
    d: Inner,
    e: [u8; 10],
    f: &'a [u8],
}

#[allow(unused)]
#[derive(Schema)]
#[postcard(bound = "")] // doesn't compile without this
struct Bound<F: bound::Fun> {
    x: F::Out<u8>,
}

mod bound {
    use super::*;

    pub trait Fun {
        type Out<In: Schema>: Schema;
    }

    pub enum Id {}
    impl Fun for Id {
        type Out<In: Schema> = In;
    }
}

#[test]
fn derive_enum() {
    assert_eq!(
        &DataModelType::Enum {
            name: "Inner'",
            variants: &[
                &Variant {
                    name: "⍺",
                    data: Data::Unit
                },
                &Variant {
                    name: "Beta",
                    data: Data::Unit
                },
                &Variant {
                    name: "Gamma",
                    data: Data::Unit
                },
                &Variant {
                    name: "Delta",
                    data: Data::Tuple(&[i32::SCHEMA, i16::SCHEMA,])
                },
                &Variant {
                    name: "Epsilon",
                    data: Data::Struct(&[
                        &NamedField {
                            name: "zeta",
                            ty: f32::SCHEMA,
                        },
                        &NamedField {
                            name: "eta",
                            ty: bool::SCHEMA,
                        }
                    ]),
                }
            ],
        },
        Inner::SCHEMA
    );
}

#[test]
fn derive_struct() {
    assert_eq!(
        Outer::SCHEMA,
        &DataModelType::Struct {
            name: "Outer'",
            data: Data::Struct(&[
                &NamedField {
                    name: "a",
                    ty: u32::SCHEMA
                },
                &NamedField {
                    name: "b'",
                    ty: u64::SCHEMA
                },
                &NamedField {
                    name: "c",
                    ty: u8::SCHEMA
                },
                &NamedField {
                    name: "d",
                    ty: Inner::SCHEMA
                },
                &NamedField {
                    name: "e",
                    ty: &DataModelType::Tuple(&[u8::SCHEMA; 10]),
                },
                &NamedField {
                    name: "f",
                    ty: &DataModelType::Seq {
                        element: u8::SCHEMA,
                        max_len: None
                    },
                },
            ]),
        }
    );
}

#[test]
fn derive_bound() {
    assert_eq!(
        &DataModelType::Struct {
            name: "Bound",
            data: Data::Struct(&[&NamedField {
                name: "x",
                ty: u8::SCHEMA
            }]),
        },
        Bound::<bound::Id>::SCHEMA,
    );
}

#[allow(unused)]
#[derive(Debug, Schema)]
enum TestEnum<'a> {
    Alpha,
    Beta(u32),
    Gamma { a: bool, b: &'a [u8] },
    Delta(f32, Option<&'a str>),
    Epsilon(TestStruct1),
}

#[allow(unused)]
#[derive(Debug, Schema)]
struct TestStruct1 {
    a: i8,
    b: i16,
    c: i32,
    d: i64,
}

#[allow(unused)]
#[derive(Debug, Schema)]
struct TestStruct2<'a> {
    x: TestEnum<'a>,
    y: TestStruct1,
    z: Result<TestStruct1, u32>,
}

#[allow(unused)]
#[derive(Debug, Schema)]
struct TestStruct3(u64);

#[allow(unused)]
#[derive(Debug, Schema)]
struct TestStruct4(u64, bool);

#[allow(unused)]
#[derive(Debug, Schema)]
enum TestEnum2 {
    Nt(u64),
    Tup(u64, bool),
}

#[test]
fn newtype_vs_tuple() {
    assert_eq!(
        TestStruct3::SCHEMA,
        &DataModelType::Struct {
            name: "TestStruct3",
            data: Data::Newtype(u64::SCHEMA)
        }
    );

    assert_eq!(
        TestStruct4::SCHEMA,
        &DataModelType::Struct {
            name: "TestStruct4",
            data: Data::Tuple(&[u64::SCHEMA, bool::SCHEMA]),
        }
    );

    assert_eq!(
        TestEnum2::SCHEMA,
        &DataModelType::Enum {
            name: "TestEnum2",
            variants: &[
                &Variant {
                    name: "Nt",
                    data: Data::Newtype(u64::SCHEMA)
                },
                &Variant {
                    name: "Tup",
                    data: Data::Tuple(&[u64::SCHEMA, bool::SCHEMA])
                },
            ],
        }
    );
}

// Formatting

fn dewit<T: Schema>() -> String {
    let schema: OwnedDataModelType = T::SCHEMA.into();
    schema.to_pseudocode()
}

#[allow(unused)]
#[derive(Schema)]
struct UnitStruct;

#[allow(unused)]
#[derive(Schema)]
struct NewTypeStruct(String);

#[allow(unused)]
#[derive(Schema)]
struct TupStruct(u64, String);

#[allow(unused)]
#[derive(Schema)]
enum Enums {
    Unit,
    Nt(u64),
    Tup(u32, bool),
}

#[allow(unused)]
#[derive(Schema)]
struct Classic {
    a: u32,
    b: u16,
    c: bool,
}

#[allow(unused)]
#[derive(Schema)]
struct ClassicGen<T: Schema> {
    a: u32,
    b: T,
}

#[test]
fn pseudocode() {
    #[allow(clippy::type_complexity)]
    let tests: &[(fn() -> String, &str)] = &[
        (dewit::<u8>, "u8"),
        (dewit::<u16>, "u16"),
        (dewit::<u32>, "u32"),
        (dewit::<u64>, "u64"),
        (dewit::<u128>, "u128"),
        (dewit::<i8>, "i8"),
        (dewit::<i16>, "i16"),
        (dewit::<i32>, "i32"),
        (dewit::<i64>, "i64"),
        (dewit::<i128>, "i128"),
        (dewit::<()>, "()"),
        (dewit::<char>, "char"),
        (dewit::<bool>, "bool"),
        (dewit::<String>, "str"),
        (dewit::<Option<u16>>, "Option<u16>"),
        (dewit::<UnitStruct>, "struct UnitStruct"),
        (dewit::<Option<UnitStruct>>, "Option<UnitStruct>"),
        (dewit::<NewTypeStruct>, "struct NewTypeStruct(str)"),
        (dewit::<Option<NewTypeStruct>>, "Option<NewTypeStruct>"),
        (
            dewit::<Enums>,
            "enum Enums { Unit, Nt(u64), Tup(u32, bool) }",
        ),
        (dewit::<Option<Enums>>, "Option<Enums>"),
        (dewit::<&[u8]>, "[u8]"),
        (dewit::<Vec<u16>>, "[u16]"),
        (dewit::<[u8; 16]>, "[u8; 16]"),
        (dewit::<(u8, u16, u32)>, "(u8, u16, u32)"),
        (dewit::<TupStruct>, "struct TupStruct(u64, str)"),
        (dewit::<Option<TupStruct>>, "Option<TupStruct>"),
        (
            dewit::<std::collections::HashMap<u32, String>>,
            "Map<u32, str>",
        ),
        (dewit::<std::collections::HashSet<u32>>, "[u32]"),
        (
            dewit::<Classic>,
            "struct Classic { a: u32, b: u16, c: bool }",
        ),
        (
            dewit::<ClassicGen<i32>>,
            "struct ClassicGen { a: u32, b: i32 }",
        ),
        (dewit::<Option<Classic>>, "Option<Classic>"),
        (dewit::<Option<ClassicGen<i32>>>, "Option<ClassicGen>"),
        (dewit::<PathBuf>, "str"),
        (dewit::<std::collections::VecDeque<u32>>, "[u32]"),
        (dewit::<std::rc::Rc<u32>>, "u32"),
        (dewit::<std::sync::Arc<u32>>, "u32"),
    ];
    for (f, s) in tests {
        assert_eq!(f().as_str(), *s);
    }
}

#[test]
fn max_size() {
    #[allow(clippy::type_complexity)]
    let tests: &[(Option<usize>, Option<usize>)] = &[
        (u8::SCHEMA.max_size(), Some(1)),
        (u16::SCHEMA.max_size(), Some(3)),
        (u32::SCHEMA.max_size(), Some(5)),
        (u64::SCHEMA.max_size(), Some(10)),
        (u128::SCHEMA.max_size(), Some(19)),
        (i8::SCHEMA.max_size(), Some(1)),
        (i16::SCHEMA.max_size(), Some(3)),
        (i32::SCHEMA.max_size(), Some(5)),
        (i64::SCHEMA.max_size(), Some(10)),
        (i128::SCHEMA.max_size(), Some(19)),
        (<()>::SCHEMA.max_size(), Some(0)),
        (char::SCHEMA.max_size(), Some(5)),
        (bool::SCHEMA.max_size(), Some(1)),
        (String::SCHEMA.max_size(), None),
        (<Option<u16>>::SCHEMA.max_size(), Some(4)),
        (UnitStruct::SCHEMA.max_size(), Some(0)),
        (<Option<UnitStruct>>::SCHEMA.max_size(), Some(1)),
        (NewTypeStruct::SCHEMA.max_size(), None),
        (<Option<NewTypeStruct>>::SCHEMA.max_size(), None),
        (Enums::SCHEMA.max_size(), Some(11)),
        (<Option<Enums>>::SCHEMA.max_size(), Some(12)),
        (<&[u8]>::SCHEMA.max_size(), None),
        (<Vec<u16>>::SCHEMA.max_size(), None),
        (<[u8; 16]>::SCHEMA.max_size(), Some(16)),
        (<(u8, u16, u32)>::SCHEMA.max_size(), Some(9)),
        (TupStruct::SCHEMA.max_size(), None),
        (<Option<TupStruct>>::SCHEMA.max_size(), None),
        (
            <std::collections::HashMap<u32, String>>::SCHEMA.max_size(),
            None,
        ),
        (<std::collections::HashSet<u32>>::SCHEMA.max_size(), None),
        (<Classic>::SCHEMA.max_size(), Some(9)),
        (<ClassicGen<i32>>::SCHEMA.max_size(), Some(10)),
        (<Option<Classic>>::SCHEMA.max_size(), Some(10)),
        (<Option<ClassicGen<i32>>>::SCHEMA.max_size(), Some(11)),
        (PathBuf::SCHEMA.max_size(), None),
    ];
    for (calc, bound) in tests {
        assert_eq!(calc, bound);
    }
}

/// `all_used_types` on each leaf kind, inside an `Option`.
#[test]
fn all_used_types_leaf_kinds() {
    use OwnedDataModelType as T;
    let leaves = [
        T::Bool,
        T::U8,
        T::Unit,
        T::Char,
        T::String { max_len: None },
        T::ByteArray { max_len: Some(4) },
    ];
    for leaf in leaves {
        let ty = T::Option(Box::new(leaf.clone()));
        let used = ty.all_used_types();
        assert_eq!(used, [ty, leaf].into_iter().collect());
    }
}

/// serde serializes `usize` and `isize` as `u64` and `i64`.
#[test]
fn usize_isize() {
    use core::num::{NonZeroIsize, NonZeroUsize};
    assert_eq!(usize::SCHEMA, &DataModelType::U64);
    assert_eq!(NonZeroUsize::SCHEMA, &DataModelType::U64);
    assert_eq!(isize::SCHEMA, &DataModelType::I64);
    assert_eq!(NonZeroIsize::SCHEMA, &DataModelType::I64);
}

#[test]
fn max_size_overflow() {
    use DataModelType::{ByteArray, Map, Option as Opt, Seq, String as Str, Tuple, U32, U8};

    const HUGE: Option<usize> = Some(usize::MAX);
    // `usize::MAX` including the length: the largest size that fits.
    const FITS: &DataModelType = &Str {
        max_len: Some(usize::MAX - (usize::BITS as usize).div_ceil(7)),
    };
    const FITS_DATA: Data = Data::Newtype(FITS);

    const OVERFLOWS: &[DataModelType] = &[
        Str { max_len: HUGE },
        ByteArray { max_len: HUGE },
        Opt(FITS),
        // The elements overflow.
        Seq {
            element: &U32,
            max_len: HUGE,
        },
        // The elements fit, but not with the length.
        Seq {
            element: &U8,
            max_len: HUGE,
        },
        Tuple(&[FITS, &U8]),
        // A key and value overflow.
        Map {
            key: FITS,
            val: &U8,
            max_len: Some(1),
        },
        // The entries overflow.
        Map {
            key: &U8,
            val: &U8,
            max_len: HUGE,
        },
        // The entries fit, but not with the length.
        Map {
            key: &U8,
            val: &DataModelType::Unit,
            max_len: HUGE,
        },
        DataModelType::Struct {
            name: "S",
            data: Data::Struct(&[
                &NamedField {
                    name: "a",
                    ty: FITS,
                },
                &NamedField { name: "b", ty: &U8 },
            ]),
        },
        DataModelType::Struct {
            name: "S",
            data: Data::Tuple(&[FITS, &U8]),
        },
        DataModelType::Enum {
            name: "E",
            variants: &[&Variant {
                name: "A",
                data: FITS_DATA,
            }],
        },
    ];

    assert_eq!(FITS.max_size(), HUGE);
    assert_eq!(OwnedDataModelType::from(FITS).max_size(), HUGE);
    assert_eq!(FITS_DATA.max_size(), HUGE);
    for ty in OVERFLOWS {
        assert_eq!(ty.max_size(), None, "{ty:?}");
        assert_eq!(OwnedDataModelType::from(ty).max_size(), None, "{ty:?}");
    }

    // Zero-size elements don't overflow.
    let units = Seq {
        element: &DataModelType::Unit,
        max_len: HUGE,
    };
    assert_eq!(units.max_size(), Some((usize::BITS as usize).div_ceil(7)));
}
