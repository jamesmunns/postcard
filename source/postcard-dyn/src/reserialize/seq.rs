use core::fmt;

use postcard_schema::schema::owned::OwnedNamedType;
use serde::{
    de::{self, DeserializeSeed, Error as _, SeqAccess},
    ser::SerializeSeq,
    Deserializer, Serializer,
};

use super::{Context, Expected};

pub struct Visitor<'a, S, Strategy> {
    pub context: &'a Context<'a, Strategy>,
    pub serializer: S,
    pub schemas: &'a [OwnedNamedType],
}

impl<'de, S, Strategy> de::Visitor<'de> for Visitor<'_, S, Strategy>
where
    S: Serializer,
    Strategy: super::Strategy,
{
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut serializer = match self.serializer.serialize_seq(seq.size_hint()) {
            Ok(serializer) => serializer,
            Err(err) => return Ok(Err(err)),
        };
        let mut seed = ElementSeed {
            context: self.context,
            serializer: &mut serializer,
            schemas: self.schemas,
            idx: 0,
        };
        while let Some(res) = seq.next_element_seed(&mut seed)? {
            match res {
                Ok(()) => {}
                Err(err) => return Ok(Err(err)),
            }
        }
        Ok(serializer.end())
    }
}

struct ElementSeed<'a, S: 'a, Strategy> {
    context: &'a Context<'a, Strategy>,
    serializer: &'a mut S,
    schemas: &'a [OwnedNamedType],
    idx: usize,
}

impl<'de, S, Strategy> DeserializeSeed<'de> for &mut ElementSeed<'_, S, Strategy>
where
    S: SerializeSeq,
    Strategy: super::Strategy,
{
    type Value = Result<(), S::Error>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let schemas = self.schemas;
        let schema = schemas.get(self.idx).ok_or_else(|| {
            D::Error::invalid_length(
                self.idx + 1,
                &Expected(format_args!("sequence of length {}", schemas.len())),
            )
        })?;
        self.context
            .reserialize_ty(schema, deserializer, |element| {
                self.serializer.serialize_element(element)
            })
    }
}
