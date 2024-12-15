//! Lossy reserialization.
//!
//! As noted [above](super), lossless serialization is only possible by compromising elsewhere.
//! This module provides lossy implementations with compromises in the serialization format:
//! - [`reserialize_with_structs_and_enums_as_maps()`] reserializes structs and enums as maps
//!   instead of actual structs and enums.

use postcard::de_flavors::Flavor;
use postcard_schema::schema::owned::OwnedNamedType;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::{reserialize, Error};

use super::{
    expecting,
    strategy::{self, Strategy as _},
    Context,
};

mod enums;
mod structs;

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
) -> Result<S::Ok, Error<postcard::Error, S::Error>>
where
    F: Flavor<'de>,
    S: Serializer,
{
    Strategy.reserialize(schema, deserializer, serializer)
}

/// Reserialize structs and enums as maps similar to [`serde_json`].
struct Strategy;

impl strategy::Strategy for Strategy {
    fn reserialize_unit_struct<'de, D: Deserializer<'de>, S: Serializer>(
        _context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        _name: &str,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        <()>::deserialize(deserializer)?;
        Ok(serializer.serialize_unit())
    }

    fn reserialize_newtype_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Struct<'_, expecting::data::Newtype>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        context.reserialize_ty(expecting.data.schema, deserializer, |inner| {
            inner.serialize(serializer)
        })
    }

    fn reserialize_tuple_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Struct<'_, expecting::data::Tuple<'_>>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        deserializer.deserialize_tuple(
            expecting.data.elements.len(),
            reserialize::tuple::Visitor {
                context,
                serializer,
                fields: expecting.data.elements,
                reserializer: expecting::Tuple,
            },
        )
    }

    fn reserialize_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Struct<'_, expecting::data::Struct<'_>>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        deserializer.deserialize_tuple(
            expecting.data.fields.len(),
            reserialize::tuple::Visitor {
                context,
                serializer,
                fields: expecting.data.fields.iter().map(|f| &f.ty),
                reserializer: structs::ReserializeStructAsMap { expecting },
            },
        )
    }

    fn reserialize_enum<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Enum<'_, '_>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        // Postcard encodes enums as (index, value)
        deserializer.deserialize_tuple(
            2,
            enums::Visitor {
                serializer,
                context,
                expecting,
            },
        )
    }
}
