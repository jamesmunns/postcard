use postcard_schema::{
    schema::{
        owned::OwnedNamedType, DataModelType, DataModelVariant, NamedType, NamedValue, NamedVariant,
    },
    Schema,
};

const U8_SCHEMA: NamedType = NamedType {
    name: "u8",
    ty: &DataModelType::U8,
};
const U32_SCHEMA: NamedType = NamedType {
    name: "u32",
    ty: &DataModelType::U32,
};
const U64_SCHEMA: NamedType = NamedType {
    name: "u64",
    ty: &DataModelType::U64,
};

const I16_SCHEMA: NamedType = NamedType {
    name: "i16",
    ty: &DataModelType::I16,
};
const I32_SCHEMA: NamedType = NamedType {
    name: "i32",
    ty: &DataModelType::I32,
};

#[allow(unused)]
#[derive(Schema)]
enum Inner {
    Alpha,
    Beta,
    Gamma,
    Delta(i32, i16),
    Epsilon { zeta: f32, eta: bool },
}

#[allow(unused)]
#[derive(Schema)]
struct Outer<'a> {
    a: u32,
    b: u64,
    c: u8,
    d: Inner,
    e: [u8; 10],
    f: &'a [u8],
}

#[allow(unused)]
#[derive(Schema)]
struct Slice<'a> {
    x: &'a [u8],
}

#[test]
fn test_enum_serialize() {
    assert_eq!(
        &NamedType {
            name: "Inner",
            ty: &DataModelType::Enum(&[
                &NamedVariant {
                    name: "Alpha",
                    ty: &DataModelVariant::UnitVariant
                },
                &NamedVariant {
                    name: "Beta",
                    ty: &DataModelVariant::UnitVariant
                },
                &NamedVariant {
                    name: "Gamma",
                    ty: &DataModelVariant::UnitVariant
                },
                &NamedVariant {
                    name: "Delta",
                    ty: &DataModelVariant::TupleVariant(&[&I32_SCHEMA, &I16_SCHEMA,])
                },
                &NamedVariant {
                    name: "Epsilon",
                    ty: &DataModelVariant::StructVariant(&[
                        &NamedValue {
                            name: "zeta",
                            ty: &NamedType {
                                name: "f32",
                                ty: &DataModelType::F32
                            },
                        },
                        &NamedValue {
                            name: "eta",
                            ty: &NamedType {
                                name: "bool",
                                ty: &DataModelType::Bool
                            },
                        }
                    ]),
                }
            ]),
        },
        Inner::SCHEMA
    );
}

#[test]
fn test_struct_serialize() {
    const TEN_BYTES_SCHEMA: &[&NamedType] = &[&U8_SCHEMA; 10];

    assert_eq!(
        Outer::SCHEMA,
        &NamedType {
            name: "Outer",
            ty: &DataModelType::Struct(&[
                &NamedValue {
                    name: "a",
                    ty: &U32_SCHEMA
                },
                &NamedValue {
                    name: "b",
                    ty: &U64_SCHEMA
                },
                &NamedValue {
                    name: "c",
                    ty: &U8_SCHEMA
                },
                &NamedValue {
                    name: "d",
                    ty: Inner::SCHEMA
                },
                &NamedValue {
                    name: "e",
                    ty: &NamedType {
                        name: "[T; N]",
                        ty: &DataModelType::Tuple(TEN_BYTES_SCHEMA),
                    }
                },
                &NamedValue {
                    name: "f",
                    ty: &NamedType {
                        name: "[T]",
                        ty: &DataModelType::Seq(&NamedType {
                            name: "u8",
                            ty: &DataModelType::U8
                        })
                    }
                },
            ]),
        }
    );
}

#[test]
fn test_slice_serialize() {
    assert_eq!(
        &NamedType {
            name: "Slice",
            ty: &DataModelType::Struct(&[&NamedValue {
                name: "x",
                ty: &NamedType {
                    name: "[T]",
                    ty: &DataModelType::Seq(&U8_SCHEMA)
                }
            },]),
        },
        Slice::SCHEMA
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

#[test]
fn owned_punning() {
    let borrowed_schema = TestStruct2::SCHEMA;
    let owned_schema: OwnedNamedType = borrowed_schema.into();

    // Check that they are the same on the wire when serialized
    let ser_borrowed_schema = postcard::to_stdvec(borrowed_schema).unwrap();
    let ser_owned_schema = postcard::to_stdvec(&owned_schema).unwrap();
    assert_eq!(ser_borrowed_schema, ser_owned_schema);

    // TODO: This is wildly repetitive, and likely could benefit from interning of
    // repeated types, strings, etc.
    assert_eq!(ser_borrowed_schema.len(), 268);

    // Check that we round-trip correctly
    let deser_borrowed_schema =
        postcard::from_bytes::<OwnedNamedType>(&ser_borrowed_schema).unwrap();
    let deser_owned_schema = postcard::from_bytes::<OwnedNamedType>(&ser_owned_schema).unwrap();
    assert_eq!(deser_borrowed_schema, deser_owned_schema);
    assert_eq!(deser_borrowed_schema, owned_schema);
    assert_eq!(deser_owned_schema, owned_schema);
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
        &NamedType {
            name: "TestStruct3",
            ty: &DataModelType::NewtypeStruct(u64::SCHEMA)
        }
    );

    assert_eq!(
        TestStruct4::SCHEMA,
        &NamedType {
            name: "TestStruct4",
            ty: &DataModelType::TupleStruct(&[u64::SCHEMA, bool::SCHEMA]),
        }
    );

    assert_eq!(
        TestEnum2::SCHEMA,
        &NamedType {
            name: "TestEnum2",
            ty: &DataModelType::Enum(&[
                &NamedVariant {
                    name: "Nt",
                    ty: &DataModelVariant::NewtypeVariant(u64::SCHEMA)
                },
                &NamedVariant {
                    name: "Tup",
                    ty: &DataModelVariant::TupleVariant(&[u64::SCHEMA, bool::SCHEMA])
                },
            ]),
        }
    );
}

// Formatting

fn dewit<T: Schema>() -> String {
    let schema: OwnedNamedType = T::SCHEMA.into();
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
fn smoke() {
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
        (dewit::<String>, "String"),
        (dewit::<Option<u16>>, "Option<u16>"),
        (dewit::<UnitStruct>, "struct UnitStruct"),
        (dewit::<Option<UnitStruct>>, "Option<UnitStruct>"),
        (dewit::<NewTypeStruct>, "struct NewTypeStruct(String)"),
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
        (dewit::<TupStruct>, "struct TupStruct(u64, String)"),
        (dewit::<Option<TupStruct>>, "Option<TupStruct>"),
        (
            dewit::<std::collections::HashMap<u32, String>>,
            "Map<u32, String>",
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
    ];
    for (f, s) in tests {
        assert_eq!(f().as_str(), *s);
    }
}
