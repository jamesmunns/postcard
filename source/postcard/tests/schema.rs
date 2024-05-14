#![cfg(feature = "experimental-derive")]

use postcard::experimental::schema::{NamedType, NamedValue, NamedVariant, Schema, SdmTy, Varint};

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

#[test]
fn test_bound_serialize() {
    assert_eq!(
        &NamedType {
            name: "Bound",
            ty: &SdmTy::Struct(&[&NamedValue {
                name: "x",
                ty: &U8_SCHEMA
            }]),
        },
        Bound::<bound::Id>::SCHEMA,
    );
}
