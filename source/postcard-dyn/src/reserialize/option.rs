use core::fmt;

use postcard_schema::schema::owned::OwnedNamedType;
use serde::{de, Deserializer, Serializer};

use super::Context;

pub struct Visitor<'a, S, Strategy> {
    pub context: &'a Context<'a, Strategy>,
    pub serializer: S,
    pub schema: &'a OwnedNamedType,
}

impl<'de, S, Strategy> de::Visitor<'de> for Visitor<'_, S, Strategy>
where
    S: Serializer,
    Strategy: super::Strategy,
{
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("option")
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(self.serializer.serialize_none())
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.context
            .reserialize_ty(self.schema, deserializer, |inner| {
                self.serializer.serialize_some(inner)
            })
    }
}
