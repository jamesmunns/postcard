use postcard_schema_ng::{
    schema::{owned::OwnedDataModelType, Data, DataModelType, NamedField, Variant},
    Schema,
};
use std::path::PathBuf;

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
        &DataModelType::Enum {
            name: "Inner",
            variants: &[
                &Variant {
                    name: "Alpha",
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
fn test_struct_serialize() {
    assert_eq!(
        Outer::SCHEMA,
        &DataModelType::Struct {
            name: "Outer",
            data: Data::Struct(&[
                &NamedField {
                    name: "a",
                    ty: u32::SCHEMA
                },
                &NamedField {
                    name: "b",
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
                    ty: &DataModelType::Array {
                        item: u8::SCHEMA,
                        count: 10
                    },
                },
                &NamedField {
                    name: "f",
                    ty: &DataModelType::Seq(u8::SCHEMA),
                },
            ]),
        }
    );
}

#[test]
fn test_slice_serialize() {
    assert_eq!(
        &DataModelType::Struct {
            name: "Slice",
            data: Data::Struct(&[&NamedField {
                name: "x",
                ty: &DataModelType::Seq(u8::SCHEMA),
            }]),
        },
        Slice::SCHEMA
    );
}

#[test]
fn test_bound_serialize() {
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

#[test]
fn owned_punning() {
    let borrowed_schema = TestStruct2::SCHEMA;
    let owned_schema: OwnedDataModelType = borrowed_schema.into();

    // Check that they are the same on the wire when serialized
    let ser_borrowed_schema = postcard::to_stdvec(borrowed_schema).unwrap();
    let ser_owned_schema = postcard::to_stdvec(&owned_schema).unwrap();
    assert_eq!(ser_borrowed_schema, ser_owned_schema);

    // TODO: This is wildly repetitive, and likely could benefit from interning of
    // repeated types, strings, etc.
    assert_eq!(ser_borrowed_schema.len(), 187);

    // Check that we round-trip correctly
    let deser_borrowed_schema =
        postcard::from_bytes::<OwnedDataModelType>(&ser_borrowed_schema).unwrap();
    let deser_owned_schema = postcard::from_bytes::<OwnedDataModelType>(&ser_owned_schema).unwrap();
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
        (dewit::<PathBuf>, "String"),
    ];
    for (f, s) in tests {
        assert_eq!(f().as_str(), *s);
    }
}
