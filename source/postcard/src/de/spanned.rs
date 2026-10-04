use crate::de_flavors::Flavor;
use crate::Deserializer;
use serde::de::value::BorrowedStrDeserializer;
use serde::de::IntoDeserializer;

pub(crate) enum SpannedDeserializer<'d, 'de, F>
where
    F: Flavor<'de>,
{
    Start {
        value_deserializer: &'d mut Deserializer<'de, F>,
    },
    Value {
        value_deserializer: &'d mut Deserializer<'de, F>,
    },
    End {
        end_pos: usize,
    },
    Done,
}

impl<'d, 'de, F> SpannedDeserializer<'d, 'de, F>
where
    F: Flavor<'de>,
{
    pub fn new(value_deserializer: &'d mut Deserializer<'de, F>) -> Self {
        Self::Start { value_deserializer }
    }
}

impl<'d, 'de, F> serde::de::MapAccess<'de> for SpannedDeserializer<'d, 'de, F>
where
    F: Flavor<'de>,
{
    type Error = <&'d mut Deserializer<'de, F> as serde::de::Deserializer<'de>>::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let key = match self {
            Self::Start { .. } => serde_spanned::__unstable::START_FIELD,
            Self::End { .. } => serde_spanned::__unstable::END_FIELD,
            Self::Value { .. } => serde_spanned::__unstable::VALUE_FIELD,
            Self::Done => return Ok(None),
        };

        seed.deserialize(BorrowedStrDeserializer::new(key))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self {
            Self::Start { .. } => {
                let prev = std::mem::replace(self, Self::Done);
                let Self::Start { value_deserializer } = prev else {
                    unreachable!()
                };

                let start = value_deserializer.pos;
                *self = Self::Value { value_deserializer };
                seed.deserialize(start.into_deserializer())
            }

            Self::Value { .. } => {
                let prev = std::mem::replace(self, Self::Done);
                let Self::Value { value_deserializer } = prev else {
                    unreachable!()
                };

                let val = seed.deserialize(&mut *value_deserializer);
                *self = Self::End {
                    end_pos: value_deserializer.pos,
                };
                val
            }

            Self::End { .. } => {
                let prev = std::mem::replace(self, Self::Done);
                let Self::End { end_pos } = prev else {
                    unreachable!()
                };
                seed.deserialize(end_pos.into_deserializer())
            }

            Self::Done => {
                panic!("should not get here");
            }
        }
    }
}
