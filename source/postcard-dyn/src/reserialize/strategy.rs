//! How to reserialize structs and enums to work around [`Deserializer`] and [`Serializer`]'s
//! `&'static str` requirements.

use postcard_schema::schema::owned::OwnedNamedType;
use serde::{Deserializer, Serialize, Serializer};

use crate::Error;

use super::{expecting, Context};

/// How to reserialize structs and enums to work around [`Deserializer`] and [`Serializer`]'s
/// `&'static str` requirements.
pub(super) trait Strategy: Sized {
    fn reserialize_unit_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        name: &str,
    ) -> Result<Result<S::Ok, S::Error>, D::Error>;

    fn reserialize_newtype_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Struct<'_, expecting::data::Newtype>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error>;

    fn reserialize_tuple_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Struct<'_, expecting::data::Tuple>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error>;

    fn reserialize_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Struct<'_, expecting::data::Struct>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error>;

    fn reserialize_enum<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Enum<'_, '_>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error>;

    fn reserialize<'de, D: Deserializer<'de>, S: Serializer>(
        &self,
        schema: &OwnedNamedType,
        deserializer: D,
        serializer: S,
    ) -> Result<S::Ok, Error<D::Error, S::Error>> {
        let context = Context { strategy: self };
        match context.reserialize_ty(schema, deserializer, |value| value.serialize(serializer)) {
            Ok(Ok(out)) => Ok(out),
            Ok(Err(err)) => Err(Error::Serialize(err)),
            Err(err) => Err(Error::Deserialize(err)),
        }
    }
}
