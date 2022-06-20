#![cfg(feature = "derive")]

use postcard::schema::{NamedType, NamedValue, NamedVariant, Schema, SdmTy, Varint};

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
}

#[allow(unused)]
#[derive(Schema)]
struct Outer {
    a: u32,
    b: u64,
    c: u8,
    d: Inner,
    e: [u8; 10],
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
            ]),
        },
        Inner::SCHEMA
    );
}

#[test]
fn test_struct_serialize() {
    assert_eq!(
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
                        ty: &SdmTy::Seq(&U8_SCHEMA),
                    }
                }
            ]),
        },
        Outer::SCHEMA
    );
}
