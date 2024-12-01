//! Lossy reserialization.
//!
//! As noted [above](super), lossless serialization is only possible by compromising elsewhere.
//! This module provides lossy implementations with compromises in the serialization format:
//! - [`reserialize_with_structs_and_enums_as_maps()`] reserializes structs and enums as maps
//!   instead of actual structs and enums.

use postcard::de_flavors::Flavor;
use postcard_schema::schema::owned::{
    OwnedDataModelVariant, OwnedNamedType, OwnedNamedValue, OwnedNamedVariant,
};
use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
    Serialize, Serializer,
};

use crate::Error;

use super::{reserialize, structs_and_enums::Reserialize};

/// Reserialize [`postcard`]-encoded data, transforming structs and enums into maps.
///
/// - Structs are transformed into maps with field names (as strings) for keys and field values
///   for values.
/// - Unit enum variants are transformed into the variant name as a string.
/// - Data-carrying (i.e., non-unit) enum variants are transformed into a single-element map
///   from the variant name (as a string) to the variant's data.
///
/// This mirrors [`serde_json`]'s behavior.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// # use postcard::ser_flavors::Flavor;
/// # use std::collections::BTreeMap;
/// use postcard_schema::Schema;
/// use serde::Serialize;
///
/// #[derive(Serialize, Schema)]
/// enum Foo {
///     Bar { a: u8, b: u8 },
/// }
///
/// let bytes = postcard::to_allocvec(&Foo::Bar { a: 5, b: 10 })?;
/// let mut serializer = postcard::Serializer {
///     output: postcard::ser_flavors::StdVec::new(),
/// };
/// postcard_dyn::reserialize::lossy::reserialize_with_structs_and_enums_as_maps(
///     &Foo::SCHEMA.into(),
///     &mut postcard::Deserializer::from_bytes(&bytes),
///     &mut serializer,
/// )?;
/// let out = serializer.output.finalize()?;
/// let deserialized: BTreeMap<&str, BTreeMap<&str, u8>> = postcard::from_bytes(&out)?;
/// assert_eq!(
///     format!("{deserialized:?}"),
///     r#"{"Bar": {"a": 5, "b": 10}}"#
/// );
/// # Ok(())
/// # }
/// ```
pub fn reserialize_with_structs_and_enums_as_maps<'de, F, S>(
    schema: &OwnedNamedType,
    deserializer: &mut postcard::Deserializer<'de, F>,
    serializer: S,
) -> Result<S::Ok, Error<S::Error>>
where
    F: Flavor<'de>,
    S: Serializer,
{
    reserialize(schema, deserializer, serializer, Strategy)
}

/// Reserialize structs and enums as maps similar to [`serde_json`].
struct Strategy;

impl super::structs_and_enums::Strategy for Strategy {
    fn serialize_unit_struct<S: Serializer>(
        &self,
        serializer: S,
        _name: &str,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit()
    }

    fn serialize_newtype_struct<S: Serializer, T: ?Sized + Serialize>(
        &self,
        serializer: S,
        _name: &str,
        value: &T,
    ) -> Result<S::Ok, S::Error> {
        value.serialize(serializer)
    }

    fn serialize_tuple_struct<S: Serializer>(
        &self,
        serializer: S,
        reserialize: impl Reserialize,
        _name: &str,
        fields: &[OwnedNamedType],
    ) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_seq(Some(fields.len()))?;
        for field in fields {
            serializer.serialize_element(&reserialize.with_schema(field))?;
        }
        serializer.end()
    }

    fn serialize_struct<S: Serializer>(
        &self,
        serializer: S,
        reserialize: impl Reserialize,
        _name: &str,
        fields: &[OwnedNamedValue],
    ) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_map(Some(fields.len()))?;
        for field in fields {
            serializer.serialize_entry(&field.name, &reserialize.with_schema(&field.ty))?;
        }
        serializer.end()
    }

    fn serialize_enum<S: Serializer>(
        &self,
        serializer: S,
        reserialize: impl Reserialize,
        _name: &str,
        _variant_index: u32,
        variant: &OwnedNamedVariant,
    ) -> Result<S::Ok, S::Error> {
        struct ReserializeTuple<'a, Reserialize> {
            reserialize: Reserialize,
            elements: &'a [OwnedNamedType],
        }
        impl<R: Reserialize> Serialize for ReserializeTuple<'_, R> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut serializer = serializer.serialize_tuple(self.elements.len())?;
                for element in self.elements {
                    serializer.serialize_element(&self.reserialize.with_schema(element))?;
                }
                serializer.end()
            }
        }

        struct ReserializeFields<'a, Reserialize> {
            reserialize: Reserialize,
            fields: &'a [OwnedNamedValue],
        }
        impl<R: Reserialize> Serialize for ReserializeFields<'_, R> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut serializer = serializer.serialize_map(Some(self.fields.len()))?;
                for field in self.fields {
                    serializer
                        .serialize_entry(&field.name, &self.reserialize.with_schema(&field.ty))?;
                }
                serializer.end()
            }
        }

        match &variant.ty {
            OwnedDataModelVariant::UnitVariant => serializer.serialize_str(&variant.name),
            OwnedDataModelVariant::NewtypeVariant(inner) => {
                let mut serializer = serializer.serialize_map(Some(1))?;
                serializer.serialize_entry(&variant.name, &reserialize.with_schema(inner))?;
                serializer.end()
            }
            OwnedDataModelVariant::TupleVariant(fields) => {
                let mut serializer = serializer.serialize_map(Some(1))?;
                serializer.serialize_entry(
                    &variant.name,
                    &ReserializeTuple {
                        reserialize,
                        elements: fields,
                    },
                )?;
                serializer.end()
            }
            OwnedDataModelVariant::StructVariant(fields) => {
                let mut serializer = serializer.serialize_map(Some(1))?;
                serializer.serialize_entry(
                    &variant.name,
                    &ReserializeFields {
                        reserialize,
                        fields,
                    },
                )?;
                serializer.end()
            }
        }
    }
}
