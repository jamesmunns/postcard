use core::fmt;

use postcard_schema::schema::owned::OwnedNamedType;
use serde::{
    de::{self, DeserializeSeed, SeqAccess},
    ser::SerializeTuple,
    Deserializer, Serializer,
};

use super::{
    expecting::{self, Unexpected},
    Context,
};

pub struct Visitor<'a, S, Strategy, Fields, Reserializer> {
    pub context: &'a Context<'a, Strategy>,
    pub serializer: S,
    pub fields: Fields,
    pub reserializer: Reserializer,
}

pub trait Reserializer<S: Serializer>: de::Expected {
    type SerializeTuple: SerializeTuple<Ok = S::Ok, Error = S::Error>;

    fn reserialize_tuple(
        &self,
        serializer: S,
        len: usize,
    ) -> Result<Self::SerializeTuple, S::Error>;
}

impl<S: Serializer> Reserializer<S> for expecting::Tuple {
    type SerializeTuple = S::SerializeTuple;

    fn reserialize_tuple(
        &self,
        serializer: S,
        len: usize,
    ) -> Result<Self::SerializeTuple, S::Error> {
        serializer.serialize_tuple(len)
    }
}

impl<'de, 'schema, S, Strategy, Fields, Reserializer> de::Visitor<'de>
    for Visitor<'_, S, Strategy, Fields, Reserializer>
where
    S: Serializer,
    Strategy: super::Strategy,
    Fields: IntoIterator<Item = &'schema OwnedNamedType, IntoIter: ExactSizeIterator>,
    Reserializer: self::Reserializer<S>,
{
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        de::Expected::fmt(&self.reserializer, formatter)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let fields = self.fields.into_iter();
        let num_fields = fields.len();
        let mut serializer = match self
            .reserializer
            .reserialize_tuple(self.serializer, num_fields)
        {
            Ok(serializer) => serializer,
            Err(err) => return Ok(Err(err)),
        };
        for (idx, field) in fields.enumerate() {
            let seed = ElementSeed {
                context: self.context,
                serializer: &mut serializer,
                field,
            };
            let res = seq
                .next_element_seed(seed)?
                .ok_or_else(|| A::Error::missing_elements(idx, &self.reserializer, num_fields))?;
            match res {
                Ok(()) => {}
                Err(err) => return Ok(Err(err)),
            }
        }
        Ok(serializer.end())
    }
}

struct ElementSeed<'a, S, Strategy> {
    context: &'a Context<'a, Strategy>,
    serializer: &'a mut S,
    field: &'a OwnedNamedType,
}

impl<'de, S, Strategy> DeserializeSeed<'de> for ElementSeed<'_, S, Strategy>
where
    S: SerializeTuple,
    Strategy: super::Strategy,
{
    type Value = Result<(), S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.context
            .reserialize_ty(self.field, deserializer, |element| {
                self.serializer.serialize_element(element)
            })
    }
}
