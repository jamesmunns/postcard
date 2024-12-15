use core::{fmt, marker::PhantomData};

use postcard_schema::schema::owned::{OwnedDataModelVariant, OwnedNamedType};
use serde::{
    de::{self, DeserializeSeed, Deserializer, SeqAccess},
    ser::{Error as _, SerializeMap, SerializeTuple, Serializer},
};

use crate::{
    reserialize::{
        expecting::{self, Unexpected},
        Context, ReserializeFn,
    },
    Error,
};

use super::Strategy;

pub struct Visitor<'a, S> {
    pub serializer: S,
    pub context: &'a Context<'a, Strategy>,
    pub expecting: expecting::Enum<'a, 'a>,
}

impl<'de, S: Serializer> de::Visitor<'de> for Visitor<'_, S> {
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        de::Expected::fmt(&self.expecting, formatter)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let variant_index: u32 = seq.next_element()?.unwrap();
        let variant = (usize::try_from(variant_index).ok())
            .and_then(|v| self.expecting.variants.get(v))
            .ok_or_else(|| {
                A::Error::unknown_variant_index(variant_index, ..self.expecting.variants.len())
            })?;

        let err = || S::Error::custom("missing variant data");
        Ok(match &variant.ty {
            OwnedDataModelVariant::UnitVariant => self.serializer.serialize_str(&variant.name),
            OwnedDataModelVariant::NewtypeVariant(inner) => seq
                .next_element_seed(NewtypeVariantSeed {
                    serializer: self.serializer,
                    context: self.context,
                    variant: &variant.name,
                    inner,
                })?
                .ok_or_else(err)
                .and_then(|res| res),
            OwnedDataModelVariant::TupleVariant(fields) => seq
                .next_element_seed(TupleVariantVisitor {
                    serializer: self.serializer,
                    context: self.context,
                    expecting: expecting::Variant {
                        enum_name: self.expecting.name,
                        variant_index,
                        variant_name: &variant.name,
                        data: expecting::data::Tuple { elements: fields },
                    },
                })?
                .ok_or_else(err)
                .and_then(|res| res),
            OwnedDataModelVariant::StructVariant(fields) => seq
                .next_element_seed(StructVariantVisitor {
                    serializer: self.serializer,
                    context: self.context,
                    expecting: expecting::Variant {
                        enum_name: self.expecting.name,
                        variant_index,
                        variant_name: &variant.name,
                        data: expecting::data::Struct { fields },
                    },
                })?
                .ok_or_else(err)
                .and_then(|res| res),
        })
    }
}

struct NewtypeVariantSeed<'a, S> {
    serializer: S,
    context: &'a Context<'a, Strategy>,
    variant: &'a str,
    inner: &'a OwnedNamedType,
}

impl<'de, S: Serializer> DeserializeSeed<'de> for NewtypeVariantSeed<'_, S> {
    type Value = Result<S::Ok, S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let mut serializer = match self.serializer.serialize_map(Some(1)) {
            Ok(serializer) => serializer,
            Err(err) => return Ok(Err(err)),
        };
        let res = self
            .context
            .reserialize_ty(self.inner, deserializer, |value| {
                serializer.serialize_entry(self.variant, value)
            })?;
        match res {
            Ok(()) => {}
            Err(err) => return Ok(Err(err)),
        }
        Ok(serializer.end())
    }
}

struct TupleVariantVisitor<'a, S> {
    serializer: S,
    context: &'a Context<'a, Strategy>,
    expecting: expecting::Variant<'a, expecting::data::Tuple<'a>>,
}

impl<'de, S: Serializer> DeserializeSeed<'de> for TupleVariantVisitor<'_, S> {
    type Value = Result<S::Ok, S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_tuple(self.expecting.data.elements.len(), self)
    }
}

impl<'de, S: Serializer> de::Visitor<'de> for TupleVariantVisitor<'_, S> {
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        de::Expected::fmt(&self.expecting, formatter)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        let mut serializer = match self.serializer.serialize_map(Some(1)) {
            Ok(serializer) => serializer,
            Err(err) => return Ok(Err(err)),
        };
        self.context.reserialize(
            ReserializeTupleVariant {
                context: self.context,
                seq,
                expecting: &self.expecting,
                de: PhantomData,
            },
            |data| {
                serializer.serialize_entry(self.expecting.variant_name, data)?;
                serializer.end()
            },
        )
    }
}
struct ReserializeTupleVariant<'de, 'a, A> {
    context: &'a Context<'a, Strategy>,
    seq: A,
    expecting: &'a expecting::Variant<'a, expecting::data::Tuple<'a>>,
    de: PhantomData<&'de ()>,
}

struct ElementSeed<'a, S> {
    context: &'a Context<'a, Strategy>,
    serializer: &'a mut S,
    schema: &'a OwnedNamedType,
}

impl<'de, S: SerializeTuple> DeserializeSeed<'de> for ElementSeed<'_, S> {
    type Value = Result<(), S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.context
            .reserialize_ty(self.schema, deserializer, |element| {
                self.serializer.serialize_element(element)
            })
    }
}

impl<'de, A: SeqAccess<'de>> ReserializeFn for ReserializeTupleVariant<'de, '_, A> {
    type DeserializeError = A::Error;

    fn reserialize<S: Serializer>(
        mut self,
        serializer: S,
    ) -> Result<S::Ok, Error<Self::DeserializeError, S::Error>> {
        let fields = self.expecting.data.elements;
        let mut serializer = serializer
            .serialize_tuple(fields.len())
            .map_err(Error::Serialize)?;
        for (idx, field) in fields.iter().enumerate() {
            self.seq
                .next_element_seed(ElementSeed {
                    context: self.context,
                    serializer: &mut serializer,
                    schema: field,
                })
                .map_err(Error::Deserialize)?
                .ok_or_else(|| A::Error::missing_elements(idx, self.expecting, fields.len()))
                .map_err(Error::Deserialize)?
                .map_err(Error::Serialize)?;
        }
        serializer.end().map_err(Error::Serialize)
    }
}

struct StructVariantVisitor<'a, S> {
    serializer: S,
    context: &'a Context<'a, Strategy>,
    expecting: expecting::Variant<'a, expecting::data::Struct<'a>>,
}

impl<'de, S: Serializer> DeserializeSeed<'de> for StructVariantVisitor<'_, S> {
    type Value = Result<S::Ok, S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_tuple(self.expecting.data.fields.len(), self)
    }
}

impl<'de, S: Serializer> de::Visitor<'de> for StructVariantVisitor<'_, S> {
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        de::Expected::fmt(&self.expecting, formatter)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        let mut serializer = match self.serializer.serialize_map(Some(1)) {
            Ok(serializer) => serializer,
            Err(err) => return Ok(Err(err)),
        };
        self.context.reserialize(
            ReserializeStructVariant {
                context: self.context,
                seq,
                expecting: &self.expecting,
                de: PhantomData,
            },
            |data| {
                serializer.serialize_entry(self.expecting.variant_name, data)?;
                serializer.end()
            },
        )
    }
}

struct ReserializeStructVariant<'a, 'de, A> {
    context: &'a Context<'a, Strategy>,
    seq: A,
    expecting: &'a expecting::Variant<'a, expecting::data::Struct<'a>>,
    de: PhantomData<&'de ()>,
}

struct FieldSeed<'a, S> {
    context: &'a Context<'a, Strategy>,
    serializer: &'a mut S,
    key: &'a str,
    schema: &'a OwnedNamedType,
}

impl<'de, S: SerializeMap> DeserializeSeed<'de> for FieldSeed<'_, S> {
    type Value = Result<(), S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.context
            .reserialize_ty(self.schema, deserializer, |value| {
                self.serializer.serialize_entry(self.key, value)
            })
    }
}

impl<'de, A: SeqAccess<'de>> ReserializeFn for ReserializeStructVariant<'_, 'de, A> {
    type DeserializeError = A::Error;

    fn reserialize<S: Serializer>(
        mut self,
        serializer: S,
    ) -> Result<S::Ok, Error<Self::DeserializeError, S::Error>> {
        let fields = self.expecting.data.fields;
        let mut serializer = serializer
            .serialize_map(Some(fields.len()))
            .map_err(Error::Serialize)?;
        for (idx, field) in fields.iter().enumerate() {
            self.seq
                .next_element_seed(FieldSeed {
                    context: self.context,
                    serializer: &mut serializer,
                    key: &field.name,
                    schema: &field.ty,
                })
                .map_err(Error::Deserialize)?
                .ok_or_else(|| A::Error::missing_elements(idx, self.expecting, fields.len()))
                .map_err(Error::Deserialize)?
                .map_err(Error::Serialize)?;
        }
        serializer.end().map_err(Error::Serialize)
    }
}
