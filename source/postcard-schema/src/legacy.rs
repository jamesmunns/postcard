//! Schemas from older versions of postcard-schema
//!
//! These let a host built with this version read the schemas that a device
//! built with an older version sends, such as the schema data that postcard-rpc
//! devices send on the `postcard-rpc/schema/data` topic.
//!
//! [`v0_2`] covers postcard-schema 0.2, as used by postcard-rpc, and
//! [`ng_v0_4`] covers postcard-schema-ng 0.4, as used by temperance.

#[cfg(feature = "postcard-schema-v0_2")]
#[cfg_attr(docsrs, doc(cfg(feature = "postcard-schema-v0_2")))]
pub mod v0_2 {
    //! The owned schema types of postcard-schema 0.2
    //!
    //! These are postcard-schema 0.2's own `schema::owned` types, re-exported
    //! from that crate. Convert an [`OwnedNamedType`] into this version's
    //! [`OwnedDataModelType`](crate::schema::OwnedDataModelType) to use it with
    //! the rest of this crate.
    //!
    //! This is a transitional path for hosts talking to older devices. It
    //! covers postcard-rpc 0.11 to 0.12, and may cover 0.8 to 0.10. It will not
    //! work with postcard-rpc 0.2 to 0.7, which compute their keys differently.
    //!
    //! To compute the key of a message that contains one of these types, use
    //! [`Flavor::PostcardSchemaV0_2`](crate::key::v2::Flavor) with the `key-v2`
    //! feature: [`OwnedNamedType`]'s schema is the `Schema` kind, as it was in
    //! 0.2.
    //!
    //! For example, a postcard-rpc 0.12 device sends `OwnedSchemaData`
    //! messages, whose `Type` variant holds this same [`OwnedNamedType`], so a
    //! host can convert the schemas it receives directly.
    //!
    //! The conversion keeps what the schema describes, including where 0.2
    //! described a type differently from this version (such as `Result`, as
    //! tuple variants of one field). Names that this version has no place
    //! for, such as the names of primitive and sequence types, are dropped.
    //! `Usize` and `Isize` become `U64` and `I64`, which have the same
    //! encoding.

    use crate::{
        schema::{DataModelType, OwnedData, OwnedNamedField, OwnedVariant},
        Schema,
    };

    use crate::schema::OwnedDataModelType as Current;

    pub use postcard_schema_v0_2::schema::owned::{
        OwnedDataModelType, OwnedDataModelVariant, OwnedNamedType, OwnedNamedValue,
        OwnedNamedVariant,
    };

    impl Schema for OwnedNamedType {
        const SCHEMA: &'static DataModelType = &DataModelType::Schema;
    }

    impl From<&OwnedNamedType> for Current {
        fn from(nt: &OwnedNamedType) -> Self {
            let name = || nt.name.as_str().into();
            let ty = |t: &OwnedNamedType| Box::new(Current::from(t));
            let tys = |ts: &[OwnedNamedType]| ts.iter().map(Current::from).collect();
            match &nt.ty {
                OwnedDataModelType::Bool => Current::Bool,
                OwnedDataModelType::I8 => Current::I8,
                OwnedDataModelType::U8 => Current::U8,
                OwnedDataModelType::I16 => Current::I16,
                OwnedDataModelType::I32 => Current::I32,
                OwnedDataModelType::I64 => Current::I64,
                OwnedDataModelType::I128 => Current::I128,
                OwnedDataModelType::U16 => Current::U16,
                OwnedDataModelType::U32 => Current::U32,
                OwnedDataModelType::U64 => Current::U64,
                OwnedDataModelType::U128 => Current::U128,
                OwnedDataModelType::Usize => Current::U64,
                OwnedDataModelType::Isize => Current::I64,
                OwnedDataModelType::F32 => Current::F32,
                OwnedDataModelType::F64 => Current::F64,
                OwnedDataModelType::Char => Current::Char,
                OwnedDataModelType::String => Current::String { max_len: None },
                OwnedDataModelType::ByteArray => Current::ByteArray { max_len: None },
                OwnedDataModelType::Option(t) => Current::Option(ty(t)),
                OwnedDataModelType::Unit => Current::Unit,
                OwnedDataModelType::UnitStruct => Current::Struct {
                    name: name(),
                    data: OwnedData::Unit,
                },
                OwnedDataModelType::NewtypeStruct(t) => Current::Struct {
                    name: name(),
                    data: OwnedData::Newtype(ty(t)),
                },
                OwnedDataModelType::Seq(t) => Current::Seq {
                    element: ty(t),
                    max_len: None,
                },
                OwnedDataModelType::Tuple(ts) => Current::Tuple(tys(ts)),
                OwnedDataModelType::TupleStruct(ts) => Current::Struct {
                    name: name(),
                    data: OwnedData::Tuple(tys(ts)),
                },
                OwnedDataModelType::Map { key, val } => Current::Map {
                    key: ty(key),
                    val: ty(val),
                    max_len: None,
                },
                OwnedDataModelType::Struct(fields) => Current::Struct {
                    name: name(),
                    data: OwnedData::Struct(fields.iter().map(field).collect()),
                },
                OwnedDataModelType::Enum(variants) => Current::Enum {
                    name: name(),
                    variants: variants.iter().map(variant).collect(),
                },
                OwnedDataModelType::Schema => Current::Schema,
            }
        }
    }

    fn field(f: &OwnedNamedValue) -> OwnedNamedField {
        OwnedNamedField {
            name: f.name.as_str().into(),
            ty: (&f.ty).into(),
        }
    }

    fn variant(v: &OwnedNamedVariant) -> OwnedVariant {
        let data = match &v.ty {
            OwnedDataModelVariant::UnitVariant => OwnedData::Unit,
            OwnedDataModelVariant::NewtypeVariant(t) => OwnedData::Newtype(Box::new((&**t).into())),
            OwnedDataModelVariant::TupleVariant(ts) => {
                OwnedData::Tuple(ts.iter().map(Current::from).collect())
            }
            OwnedDataModelVariant::StructVariant(fields) => {
                OwnedData::Struct(fields.iter().map(field).collect())
            }
        };
        OwnedVariant {
            name: v.name.as_str().into(),
            data,
        }
    }
}

#[cfg(feature = "postcard-schema-ng-v0_4")]
#[cfg_attr(docsrs, doc(cfg(feature = "postcard-schema-ng-v0_4")))]
pub mod ng_v0_4 {
    //! The owned schema types of postcard-schema-ng 0.4
    //!
    //! These are postcard-schema-ng 0.4's own `schema::owned` types,
    //! re-exported from that crate. Convert an [`OwnedDataModelType`] into this
    //! version's [`OwnedDataModelType`](crate::schema::OwnedDataModelType) to
    //! use it with the rest of this crate.
    //!
    //! This is a transitional path for hosts talking to older devices. It
    //! covers nprpc 1.0 to 2.1 and temperance 2.1 to 2.2, which compute method
    //! keys with the maximum lengths option enabled.
    //!
    //! The two are the same shape, except that this version has no `Usize` and
    //! `Isize` kinds, which become `U64` and `I64`, with the same encoding. As
    //! that shifts the variants after them, a schema encoded by
    //! postcard-schema-ng 0.4 must be decoded as this module's type, not as
    //! this version's.
    //!
    //! To compute the key of a message that contains one of these types, use
    //! [`Flavor::PostcardSchemaNgV0_4`](crate::key::v2::Flavor) with the
    //! `key-v2` feature: [`OwnedDataModelType`]'s schema is the `Schema` kind.

    use crate::{
        schema::{DataModelType, OwnedData as Data, OwnedNamedField as Field, OwnedVariant as Var},
        Schema,
    };

    use crate::schema::OwnedDataModelType as Current;

    pub use postcard_schema_ng_v0_4::schema::owned::{
        OwnedData, OwnedDataModelType, OwnedNamedField, OwnedVariant,
    };

    impl Schema for OwnedDataModelType {
        const SCHEMA: &'static DataModelType = &DataModelType::Schema;
    }

    impl From<&OwnedDataModelType> for Current {
        fn from(ty: &OwnedDataModelType) -> Self {
            use OwnedDataModelType as Ng;
            let b = |t: &OwnedDataModelType| Box::new(Current::from(t));
            match ty {
                Ng::Bool => Current::Bool,
                Ng::I8 => Current::I8,
                Ng::U8 => Current::U8,
                Ng::I16 => Current::I16,
                Ng::I32 => Current::I32,
                Ng::I64 | Ng::Isize => Current::I64,
                Ng::I128 => Current::I128,
                Ng::U16 => Current::U16,
                Ng::U32 => Current::U32,
                Ng::U64 | Ng::Usize => Current::U64,
                Ng::U128 => Current::U128,
                Ng::F32 => Current::F32,
                Ng::F64 => Current::F64,
                Ng::Char => Current::Char,
                Ng::String { max_len } => Current::String { max_len: *max_len },
                Ng::ByteArray { max_len } => Current::ByteArray { max_len: *max_len },
                Ng::Option(t) => Current::Option(b(t)),
                Ng::Unit => Current::Unit,
                Ng::Seq { element, max_len } => Current::Seq {
                    element: b(element),
                    max_len: *max_len,
                },
                Ng::Tuple(ts) => Current::Tuple(ts.iter().map(Current::from).collect()),
                Ng::Map { key, val, max_len } => Current::Map {
                    key: b(key),
                    val: b(val),
                    max_len: *max_len,
                },
                Ng::Struct { name, data } => Current::Struct {
                    name: name.clone(),
                    data: data.into(),
                },
                Ng::Enum { name, variants } => Current::Enum {
                    name: name.clone(),
                    variants: variants
                        .iter()
                        .map(|v| Var {
                            name: v.name.clone(),
                            data: (&v.data).into(),
                        })
                        .collect(),
                },
                Ng::Schema => Current::Schema,
            }
        }
    }

    impl From<&OwnedData> for Data {
        fn from(data: &OwnedData) -> Self {
            match data {
                OwnedData::Unit => Data::Unit,
                OwnedData::Newtype(t) => Data::Newtype(Box::new((&**t).into())),
                OwnedData::Tuple(ts) => Data::Tuple(ts.iter().map(Current::from).collect()),
                OwnedData::Struct(fs) => Data::Struct(
                    fs.iter()
                        .map(|f| Field {
                            name: f.name.clone(),
                            ty: (&f.ty).into(),
                        })
                        .collect(),
                ),
            }
        }
    }
}
