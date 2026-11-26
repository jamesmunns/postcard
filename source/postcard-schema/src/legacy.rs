//! Keys and schemas of older versions of postcard-schema
//!
//! These are for talking to peers built with an older version. The rest
//! of this crate is the current version, which new code should use.
//!
//! [`key_v2`] computes the version 2 keys of postcard-schema 0.2, as used by
//! postcard-rpc, and postcard-schema-ng 0.4, as used by temperance.
//!
//! These let a host built with this version read the schemas that a device
//! built with an older version sends, such as the schema data that postcard-rpc
//! devices send on the `postcard-rpc/schema/data` topic.
//!
//! [`v0_2`] covers postcard-schema 0.2, as used by postcard-rpc, and
//! [`ng_v0_4`] covers postcard-schema-ng 0.4, as used by temperance.

#[cfg(feature = "legacy-key-v2")]
#[cfg_attr(docsrs, doc(cfg(feature = "legacy-key-v2")))]
pub mod key_v2;

/// A legacy schema contains the `Schema` kind, which this version doesn't have
///
/// A schema is now encoded as a schema table, an ordinary type, so there is
/// no kind meaning "a schema".
#[cfg(any(feature = "legacy-schema-v0_2", feature = "legacy-schema-ng-v0_4"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaKindUnsupported;

#[cfg(any(feature = "legacy-schema-v0_2", feature = "legacy-schema-ng-v0_4"))]
impl core::fmt::Display for SchemaKindUnsupported {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("the `Schema` kind is not supported")
    }
}

#[cfg(any(feature = "legacy-schema-v0_2", feature = "legacy-schema-ng-v0_4"))]
impl std::error::Error for SchemaKindUnsupported {}

#[cfg(feature = "legacy-schema-v0_2")]
#[cfg_attr(docsrs, doc(cfg(feature = "legacy-schema-v0_2")))]
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
    //! The key of a message that contains one of these types, such as the
    //! `postcard-rpc/schema/data` topic, can't be computed with this version,
    //! as it has no `Schema` kind; use the old key with
    //! [`Key::from_bytes`](crate::key::Key::from_bytes).
    //!
    //! For example, a postcard-rpc 0.12 device sends `OwnedSchemaData`
    //! messages, whose `Type` variant holds this same [`OwnedNamedType`], so a
    //! host can convert the schemas it receives directly.

    use super::SchemaKindUnsupported as Unsupported;
    use crate::schema::{OwnedData, OwnedNamedField, OwnedVariant};

    use crate::schema::OwnedDataModelType as Current;

    pub use postcard_schema_v0_2::schema::owned::{
        OwnedDataModelType, OwnedDataModelVariant, OwnedNamedType, OwnedNamedValue,
        OwnedNamedVariant,
    };

    impl TryFrom<&OwnedNamedType> for Current {
        type Error = Unsupported;

        fn try_from(nt: &OwnedNamedType) -> Result<Self, Unsupported> {
            let name = || nt.name.as_str().into();
            let ty = |t: &OwnedNamedType| Current::try_from(t).map(Box::new);
            let tys = |ts: &[OwnedNamedType]| {
                ts.iter()
                    .map(Current::try_from)
                    .collect::<Result<Box<[_]>, _>>()
            };
            Ok(match &nt.ty {
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
                OwnedDataModelType::Option(t) => Current::Option(ty(t)?),
                OwnedDataModelType::Unit => Current::Unit,
                OwnedDataModelType::UnitStruct => Current::Struct {
                    name: name(),
                    data: OwnedData::Unit,
                },
                OwnedDataModelType::NewtypeStruct(t) => Current::Struct {
                    name: name(),
                    data: OwnedData::Newtype(ty(t)?),
                },
                OwnedDataModelType::Seq(t) => Current::Seq {
                    element: ty(t)?,
                    max_len: None,
                },
                OwnedDataModelType::Tuple(ts) => Current::Tuple(tys(ts)?),
                OwnedDataModelType::TupleStruct(ts) => Current::Struct {
                    name: name(),
                    data: OwnedData::Tuple(tys(ts)?),
                },
                OwnedDataModelType::Map { key, val } => Current::Map {
                    key: ty(key)?,
                    val: ty(val)?,
                    max_len: None,
                },
                OwnedDataModelType::Struct(fields) => Current::Struct {
                    name: name(),
                    data: OwnedData::Struct(fields.iter().map(field).collect::<Result<_, _>>()?),
                },
                OwnedDataModelType::Enum(variants) => Current::Enum {
                    name: name(),
                    variants: variants.iter().map(variant).collect::<Result<_, _>>()?,
                },
                OwnedDataModelType::Schema => return Err(Unsupported),
            })
        }
    }

    fn field(f: &OwnedNamedValue) -> Result<OwnedNamedField, Unsupported> {
        Ok(OwnedNamedField {
            name: f.name.as_str().into(),
            ty: (&f.ty).try_into()?,
        })
    }

    fn variant(v: &OwnedNamedVariant) -> Result<OwnedVariant, Unsupported> {
        let data = match &v.ty {
            OwnedDataModelVariant::UnitVariant => OwnedData::Unit,
            OwnedDataModelVariant::NewtypeVariant(t) => {
                OwnedData::Newtype(Box::new((&**t).try_into()?))
            }
            OwnedDataModelVariant::TupleVariant(ts) => {
                OwnedData::Tuple(ts.iter().map(Current::try_from).collect::<Result<_, _>>()?)
            }
            OwnedDataModelVariant::StructVariant(fields) => {
                OwnedData::Struct(fields.iter().map(field).collect::<Result<_, _>>()?)
            }
        };
        Ok(OwnedVariant {
            name: v.name.as_str().into(),
            data,
        })
    }
}

#[cfg(feature = "legacy-schema-ng-v0_4")]
#[cfg_attr(docsrs, doc(cfg(feature = "legacy-schema-ng-v0_4")))]
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
    //! The two are mostly similar, except that this version has no `Usize` and
    //! `Isize` kinds, which become `U64` and `I64`, with the same encoding.
    //! A schema encoded by postcard-schema-ng 0.4 is decoded as this module's
    //! type, then converted.

    use super::SchemaKindUnsupported as Unsupported;
    use crate::schema::{OwnedData as Data, OwnedNamedField as Field, OwnedVariant as Var};

    use crate::schema::OwnedDataModelType as Current;

    pub use postcard_schema_ng_v0_4::schema::owned::{
        OwnedData, OwnedDataModelType, OwnedNamedField, OwnedVariant,
    };

    impl TryFrom<&OwnedDataModelType> for Current {
        type Error = Unsupported;

        fn try_from(ty: &OwnedDataModelType) -> Result<Self, Unsupported> {
            use OwnedDataModelType as Ng;
            let b = |t: &OwnedDataModelType| Current::try_from(t).map(Box::new);
            Ok(match ty {
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
                Ng::Option(t) => Current::Option(b(t)?),
                Ng::Unit => Current::Unit,
                Ng::Seq { element, max_len } => Current::Seq {
                    element: b(element)?,
                    max_len: *max_len,
                },
                Ng::Tuple(ts) => {
                    Current::Tuple(ts.iter().map(Current::try_from).collect::<Result<_, _>>()?)
                }
                Ng::Map { key, val, max_len } => Current::Map {
                    key: b(key)?,
                    val: b(val)?,
                    max_len: *max_len,
                },
                Ng::Struct { name, data } => Current::Struct {
                    name: name.clone(),
                    data: data.try_into()?,
                },
                Ng::Enum { name, variants } => Current::Enum {
                    name: name.clone(),
                    variants: variants
                        .iter()
                        .map(|v| {
                            Ok(Var {
                                name: v.name.clone(),
                                data: (&v.data).try_into()?,
                            })
                        })
                        .collect::<Result<_, Unsupported>>()?,
                },
                Ng::Schema => return Err(Unsupported),
            })
        }
    }

    impl TryFrom<&OwnedData> for Data {
        type Error = Unsupported;

        fn try_from(data: &OwnedData) -> Result<Self, Unsupported> {
            Ok(match data {
                OwnedData::Unit => Data::Unit,
                OwnedData::Newtype(t) => Data::Newtype(Box::new((&**t).try_into()?)),
                OwnedData::Tuple(ts) => {
                    Data::Tuple(ts.iter().map(Current::try_from).collect::<Result<_, _>>()?)
                }
                OwnedData::Struct(fs) => Data::Struct(
                    fs.iter()
                        .map(|f| {
                            Ok(Field {
                                name: f.name.clone(),
                                ty: (&f.ty).try_into()?,
                            })
                        })
                        .collect::<Result<_, Unsupported>>()?,
                ),
            })
        }
    }
}
