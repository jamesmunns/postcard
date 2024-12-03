//! Dynamically reserialize [`postcard`]-encoded values into a [`Serializer`].
//!
//! This module implements transformations from postcard-encoded data to other serialized forms
//! based on [dynamic schemas](postcard_schema::schema::owned). For example, this could be used to
//! transform postcard-encoded data to JSON or another human-readable format.
//!
//! # Limitations
//!
//! Several [`Serializer`] methods require `&'static str`s parameters: namely, those for
//! serializing structs and enums require `&'static str`s for the names of structs, fields, enums,
//! and variants. Since these transformations work with dynamic schemas that contain [`String`]s
//! instead of `&'static str`s, lossless reserialization is possible only with other compromises.
//!
//! In particular, reserialization can be either:
//! - Lossless with implementation compromises: see [`lossless`]
//! - Lossy with regards to structs and enums: see [`lossy`]

use core::cell::{Cell, RefCell};

use postcard::{de_flavors::Flavor, Deserializer};
use postcard_schema::schema::owned::{OwnedDataModelType, OwnedNamedType};
use serde::{
    ser::{Error as _, SerializeMap, SerializeSeq, SerializeTuple},
    Deserialize, Serialize, Serializer,
};

use crate::Error;

pub mod lossless;
pub mod lossy;

fn reserialize<'de, F, S>(
    schema: &OwnedNamedType,
    deserializer: &mut Deserializer<'de, F>,
    serializer: S,
    structs_and_enums: impl structs_and_enums::Strategy,
) -> Result<S::Ok, Error<S::Error>>
where
    F: Flavor<'de>,
    S: Serializer,
{
    let reserializer = Reserialize {
        schema,
        deserializer: &RefCell::new(deserializer),
        deserializer_error: &Cell::new(None),
        strategy: &structs_and_enums,
    };
    match reserializer.serialize(serializer) {
        Ok(out) => {
            debug_assert_eq!(reserializer.deserializer_error.take(), None);
            Ok(out)
        }
        Err(err) => {
            if let Some(err) = reserializer.deserializer_error.take() {
                Err(Error::Deserialize(err))
            } else {
                Err(Error::Serialize(err))
            }
        }
    }
}

struct Reserialize<'a, 'de, 'deserializer, F, Strategy>
where
    F: Flavor<'de> + 'de,
{
    schema: &'a OwnedNamedType,
    deserializer: &'a RefCell<&'deserializer mut postcard::Deserializer<'de, F>>,
    deserializer_error: &'a Cell<Option<postcard::Error>>,
    strategy: &'a Strategy,
}

impl<'a, 'de, 'deserializer, F, Strategy> serde::Serialize
    for Reserialize<'a, 'de, 'deserializer, F, Strategy>
where
    F: Flavor<'de> + 'de,
    Strategy: structs_and_enums::Strategy,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.serialize_inner(serializer) {
            Ok(out) => Ok(out),
            Err(Error::Serialize(err)) => Err(err),
            Err(Error::Deserialize(err)) => {
                self.deserializer_error.set(Some(err.clone()));
                Err(S::Error::custom(err))
            }
        }
    }
}

impl<'a, 'de: 'a, 'deserializer, F, Strategy> Reserialize<'a, 'de, 'deserializer, F, Strategy>
where
    F: Flavor<'de>,
    Strategy: structs_and_enums::Strategy,
{
    fn serialize_inner<S: Serializer>(&self, serializer: S) -> Result<S::Ok, Error<S::Error>> {
        match &self.schema.ty {
            OwnedDataModelType::Bool => serializer.serialize_bool(self.deserialize()?),
            OwnedDataModelType::U8 => serializer.serialize_u8(self.deserialize()?),
            OwnedDataModelType::U16 => serializer.serialize_u16(self.deserialize()?),
            OwnedDataModelType::U32 => serializer.serialize_u32(self.deserialize()?),
            OwnedDataModelType::U64 => serializer.serialize_u64(self.deserialize()?),
            OwnedDataModelType::U128 => serializer.serialize_u128(self.deserialize()?),
            OwnedDataModelType::I8 => serializer.serialize_i8(self.deserialize()?),
            OwnedDataModelType::I16 => serializer.serialize_i16(self.deserialize()?),
            OwnedDataModelType::I32 => serializer.serialize_i32(self.deserialize()?),
            OwnedDataModelType::I64 => serializer.serialize_i64(self.deserialize()?),
            OwnedDataModelType::I128 => serializer.serialize_i128(self.deserialize()?),
            OwnedDataModelType::Usize => self.deserialize::<usize, _>()?.serialize(serializer),
            OwnedDataModelType::Isize => self.deserialize::<isize, _>()?.serialize(serializer),
            OwnedDataModelType::F32 => serializer.serialize_f32(self.deserialize()?),
            OwnedDataModelType::F64 => serializer.serialize_f64(self.deserialize()?),
            OwnedDataModelType::Char => serializer.serialize_char(self.deserialize()?),
            OwnedDataModelType::String => serializer.serialize_str(self.deserialize()?),
            OwnedDataModelType::ByteArray => serializer.serialize_bytes(self.deserialize()?),
            OwnedDataModelType::Option(inner) => {
                if self.deserialize()? {
                    serializer.serialize_some(&self.with_schema(inner))
                } else {
                    serializer.serialize_none()
                }
            }
            OwnedDataModelType::Unit => serializer.serialize_unit(),
            OwnedDataModelType::Seq(element) => {
                let len = self.deserialize()?;
                serializer
                    .serialize_seq(Some(len))
                    .and_then(|mut serializer| {
                        for _ in 0..len {
                            serializer.serialize_element(&self.with_schema(element))?;
                        }
                        serializer.end()
                    })
            }
            OwnedDataModelType::Tuple(elements) => serializer
                .serialize_tuple(elements.len())
                .and_then(|mut serializer| {
                    for element in elements {
                        serializer.serialize_element(&self.with_schema(element))?;
                    }
                    serializer.end()
                }),
            OwnedDataModelType::UnitStruct => self
                .strategy
                .serialize_unit_struct(serializer, &self.schema.name),
            OwnedDataModelType::NewtypeStruct(inner) => self.strategy.serialize_newtype_struct(
                serializer,
                &self.schema.name,
                &self.with_schema(inner),
            ),
            OwnedDataModelType::TupleStruct(fields) => {
                self.strategy
                    .serialize_tuple_struct(serializer, self, &self.schema.name, fields)
            }
            OwnedDataModelType::Struct(fields) => {
                self.strategy
                    .serialize_struct(serializer, self, &self.schema.name, fields)
            }
            OwnedDataModelType::Map { key, val } => {
                let map_len = self.deserialize()?;
                serializer
                    .serialize_map(Some(map_len))
                    .and_then(|mut serializer| {
                        for _ in 0..map_len {
                            // Important these are deserialized in order instead of using
                            // serialize_entry() which could deserialize the value first
                            serializer.serialize_key(&self.with_schema(key))?;
                            serializer.serialize_value(&self.with_schema(val))?;
                        }
                        serializer.end()
                    })
            }
            OwnedDataModelType::Enum(variants) => {
                let variant: u32 = self.deserialize()?;
                let schema = usize::try_from(variant)
                    .ok()
                    .and_then(|variant| variants.get(variant))
                    .ok_or(postcard::Error::DeserializeBadEncoding)
                    .map_err(Error::Deserialize)?;
                self.strategy
                    .serialize_enum(serializer, self, &self.schema.name, variant, schema)
            }
            OwnedDataModelType::Schema => todo!(),
        }
        .map_err(Error::Serialize)
    }

    fn with_schema(&self, schema: &'a OwnedNamedType) -> Self {
        Self {
            schema,
            deserializer: self.deserializer,
            deserializer_error: self.deserializer_error,
            strategy: self.strategy,
        }
    }

    fn deserialize<T: Deserialize<'de>, SerializeError: serde::ser::Error>(
        &self,
    ) -> Result<T, Error<SerializeError>> {
        T::deserialize(&mut **self.deserializer.borrow_mut()).map_err(Error::Deserialize)
    }
}

mod structs_and_enums {
    //! How to reserialize structs and enums to work around [`Serializer`]'s `&'static str` requirements.

    use postcard::de_flavors::Flavor;
    use postcard_schema::schema::owned::{OwnedNamedType, OwnedNamedValue, OwnedNamedVariant};
    use serde::{Serialize, Serializer};

    /// Type-erased wrapper around [`super::Reserialize`] to avoid needing to introduce all of the
    /// generic lifetime parameters and bounds associated with the aforementioned.
    pub(crate) trait Reserialize {
        fn with_schema<'a>(&'a self, schema: &'a OwnedNamedType) -> impl Serialize + 'a;
    }

    impl<'a, 'de, 'deserializer, F, Strategy> Reserialize
        for &super::Reserialize<'a, 'de, 'deserializer, F, Strategy>
    where
        F: Flavor<'de> + 'de,
        Strategy: self::Strategy,
    {
        fn with_schema<'b>(&'b self, schema: &'b OwnedNamedType) -> impl Serialize + 'b {
            super::Reserialize::with_schema(self, schema)
        }
    }

    /// How to reserialize structs and enums to work around [`Serializer`]'s `&'static str` requirements.
    pub(crate) trait Strategy {
        fn serialize_unit_struct<S: Serializer>(
            &self,
            serializer: S,
            name: &str,
        ) -> Result<S::Ok, S::Error>;

        fn serialize_newtype_struct<S: Serializer, T: ?Sized + Serialize>(
            &self,
            serializer: S,
            name: &str,
            value: &T,
        ) -> Result<S::Ok, S::Error>;

        fn serialize_tuple_struct<S: Serializer>(
            &self,
            serializer: S,
            reserialize: impl Reserialize,
            name: &str,
            fields: &[OwnedNamedType],
        ) -> Result<S::Ok, S::Error>;

        fn serialize_struct<S: Serializer>(
            &self,
            serializer: S,
            reserialize: impl Reserialize,
            name: &str,
            fields: &[OwnedNamedValue],
        ) -> Result<S::Ok, S::Error>;

        fn serialize_enum<S: Serializer>(
            &self,
            serializer: S,
            reserialize: impl Reserialize,
            name: &str,
            variant_index: u32,
            variant: &OwnedNamedVariant,
        ) -> Result<S::Ok, S::Error>;
    }
}

#[test]
fn errors() {
    use postcard_schema::Schema;

    assert!(matches!(
        lossy::reserialize_with_structs_and_enums_as_maps(
            &u8::SCHEMA.into(),
            &mut postcard::Deserializer::from_bytes(&[]),
            serde_json::value::Serializer
        ),
        Err(Error::Deserialize(
            postcard::Error::DeserializeUnexpectedEnd
        ))
    ));
    // Bad enum discriminant
    assert!(matches!(
        lossy::reserialize_with_structs_and_enums_as_maps(
            &Result::<u8, ()>::SCHEMA.into(),
            &mut postcard::Deserializer::from_bytes(&[99]),
            serde_json::value::Serializer
        ),
        Err(Error::Deserialize(postcard::Error::DeserializeBadEncoding))
    ));
}
