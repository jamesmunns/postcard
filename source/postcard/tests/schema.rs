#![cfg(feature = "experimental-derive")]

use postcard::experimental::schema::{
    NamedType, NamedValue, NamedVariant, OwnedNamedType, Schema, SdmTy, Varint,
};

const U8_SCHEMA: NamedType = NamedType {
    name: "u8",
    ty: &SdmTy::U8,
};
const U32_SCHEMA: NamedType = NamedType {
    name: "u32",
    ty: &SdmTy::Varint(Varint::U32),
};
const U64_SCHEMA: NamedType = NamedType {
    name: "u64",
    ty: &SdmTy::Varint(Varint::U64),
};

const I16_SCHEMA: NamedType = NamedType {
    name: "i16",
    ty: &SdmTy::Varint(Varint::I16),
};
const I32_SCHEMA: NamedType = NamedType {
    name: "i32",
    ty: &SdmTy::Varint(Varint::I32),
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
            ty: &SdmTy::Enum(&[
                &NamedVariant {
                    name: "Alpha",
                    ty: &SdmTy::UnitVariant
                },
                &NamedVariant {
                    name: "Beta",
                    ty: &SdmTy::UnitVariant
                },
                &NamedVariant {
                    name: "Gamma",
                    ty: &SdmTy::UnitVariant
                },
                &NamedVariant {
                    name: "Delta",
                    ty: &SdmTy::TupleVariant(&[&I32_SCHEMA, &I16_SCHEMA,])
                },
                &NamedVariant {
                    name: "Epsilon",
                    ty: &SdmTy::StructVariant(&[
                        &NamedValue {
                            name: "zeta",
                            ty: &NamedType {
                                name: "f32",
                                ty: &SdmTy::F32
                            },
                        },
                        &NamedValue {
                            name: "eta",
                            ty: &NamedType {
                                name: "bool",
                                ty: &SdmTy::Bool
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
            ty: &SdmTy::Struct(&[
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
                        ty: &SdmTy::Tuple(TEN_BYTES_SCHEMA),
                    }
                },
                &NamedValue {
                    name: "f",
                    ty: &NamedType {
                        name: "&[T]",
                        ty: &SdmTy::Seq(&NamedType {
                            name: "u8",
                            ty: &SdmTy::U8
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
            ty: &SdmTy::Struct(&[&NamedValue {
                name: "x",
                ty: &NamedType {
                    name: "&[T]",
                    ty: &SdmTy::Seq(&U8_SCHEMA)
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
    assert_eq!(ser_borrowed_schema.len(), 280);

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
            ty: &SdmTy::NewtypeStruct(u64::SCHEMA)
        }
    );

    assert_eq!(
        TestStruct4::SCHEMA,
        &NamedType {
            name: "TestStruct4",
            ty: &SdmTy::TupleStruct(&[u64::SCHEMA, bool::SCHEMA]),
        }
    );

    assert_eq!(
        TestEnum2::SCHEMA,
        &NamedType {
            name: "TestEnum2",
            ty: &SdmTy::Enum(&[
                &NamedVariant {
                    name: "Nt",
                    ty: &SdmTy::NewtypeVariant(u64::SCHEMA)
                },
                &NamedVariant {
                    name: "Tup",
                    ty: &SdmTy::TupleVariant(&[u64::SCHEMA, bool::SCHEMA])
                },
            ]),
        }
    );
}
