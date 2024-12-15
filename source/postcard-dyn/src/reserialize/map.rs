use core::fmt;

use postcard_schema::schema::owned::OwnedNamedType;
use serde::{
    de::{self, DeserializeSeed, MapAccess},
    ser::SerializeMap,
    Deserializer, Serializer,
};

use super::Context;

pub struct Visitor<'a, S, Strategy> {
    pub context: &'a Context<'a, Strategy>,
    pub serializer: S,
    pub key: &'a OwnedNamedType,
    pub val: &'a OwnedNamedType,
}

impl<'de, S, Strategy> de::Visitor<'de> for Visitor<'_, S, Strategy>
where
    S: Serializer,
    Strategy: super::Strategy,
{
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut serializer = match self.serializer.serialize_map(map.size_hint()) {
            Ok(serializer) => serializer,
            Err(err) => return Ok(Err(err)),
        };
        while let Some(res) = map.next_key_seed(KeySeed {
            context: self.context,
            serializer: &mut serializer,
            schema: self.key,
        })? {
            match res {
                Ok(()) => {}
                Err(err) => return Ok(Err(err)),
            }
            let seed = ValueSeed {
                context: self.context,
                serializer: &mut serializer,
                schema: self.val,
            };
            match map.next_value_seed(seed)? {
                Ok(()) => {}
                Err(err) => return Ok(Err(err)),
            }
        }
        Ok(serializer.end())
    }
}

struct KeySeed<'a, S, Strategy> {
    context: &'a Context<'a, Strategy>,
    serializer: &'a mut S,
    schema: &'a OwnedNamedType,
}

impl<'de, S, Strategy> DeserializeSeed<'de> for KeySeed<'_, S, Strategy>
where
    S: SerializeMap,
    Strategy: super::Strategy,
{
    type Value = Result<(), S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.context
            .reserialize_ty(self.schema, deserializer, |key| {
                self.serializer.serialize_key(key)
            })
    }
}

struct ValueSeed<'a, S, Strategy> {
    context: &'a Context<'a, Strategy>,
    serializer: &'a mut S,
    schema: &'a OwnedNamedType,
}

impl<'de, S, Strategy> DeserializeSeed<'de> for ValueSeed<'_, S, Strategy>
where
    S: SerializeMap,
    Strategy: super::Strategy,
{
    type Value = Result<(), S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.context
            .reserialize_ty(self.schema, deserializer, |val| {
                self.serializer.serialize_value(val)
            })
    }
}
