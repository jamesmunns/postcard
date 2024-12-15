use core::{fmt, str};

use postcard_schema::schema::owned::{OwnedDataModelVariant, OwnedNamedType, OwnedNamedVariant};
use serde::{
    de::{self, DeserializeSeed, EnumAccess, VariantAccess},
    Deserializer, Serializer,
};

use crate::reserialize::{
    self,
    expecting::{self, Unexpected},
    Context,
};

use super::Strategy;

pub struct Visitor<'a, S> {
    pub context: &'a Context<'a, Strategy>,
    pub serializer: S,
    pub expecting: expecting::Enum<'static, 'a>,
    pub variant_names: &'static [&'static str],
}

impl<'de, S: Serializer> de::Visitor<'de> for Visitor<'_, S> {
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        de::Expected::fmt(&self.expecting, formatter)
    }

    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
        let ((variant_index, variant_name, variant), deserializer) =
            data.variant_seed(VariantVisitor {
                variants: self.expecting.variants,
                variant_names: self.variant_names,
            })?;
        match variant {
            OwnedDataModelVariant::UnitVariant => {
                deserializer.unit_variant()?;
                Ok(self.serializer.serialize_unit_variant(
                    self.expecting.name,
                    variant_index,
                    variant_name,
                ))
            }
            OwnedDataModelVariant::NewtypeVariant(inner) => {
                deserializer.newtype_variant_seed(NewtypeVariantSeed {
                    context: self.context,
                    schema: inner,
                    serializer: self.serializer,
                    location: expecting::Variant {
                        enum_name: self.expecting.name,
                        variant_index,
                        variant_name,
                        data: expecting::data::Newtype { schema: inner },
                    },
                })
            }
            OwnedDataModelVariant::TupleVariant(fields) => deserializer.tuple_variant(
                fields.len(),
                reserialize::tuple::Visitor {
                    context: self.context,
                    serializer: self.serializer,
                    fields,
                    reserializer: expecting::Variant {
                        enum_name: self.expecting.name,
                        variant_index,
                        variant_name,
                        data: expecting::data::Tuple { elements: fields },
                    },
                },
            ),
            OwnedDataModelVariant::StructVariant(fields) => {
                let field_names = self.context.strategy.with_interned(|interned| {
                    interned.intern_slice(fields.iter().map(|f| f.name.as_str()))
                });
                deserializer.struct_variant(
                    field_names,
                    super::structs::Visitor {
                        context: self.context,
                        serializer: self.serializer,
                        fields,
                        field_names,
                        reserializer: expecting::Variant {
                            enum_name: self.expecting.name,
                            variant_index,
                            variant_name,
                            data: expecting::data::Struct { fields },
                        },
                    },
                )
            }
        }
    }
}

struct VariantVisitor<'a> {
    variants: &'a [OwnedNamedVariant],
    variant_names: &'static [&'static str],
}

impl<'a, 'de> DeserializeSeed<'de> for VariantVisitor<'a> {
    type Value = (u32, &'static str, &'a OwnedDataModelVariant);

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_identifier(self)
    }
}

impl<'a> de::Visitor<'_> for VariantVisitor<'a> {
    type Value = (u32, &'static str, &'a OwnedDataModelVariant);

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "variant identifier")
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        let err = || E::unknown_variant_index(value, ..self.variants.len());
        let index = u32::try_from(value).map_err(|_| err())?;
        let (name, schema) = {
            let idx = usize::try_from(value).map_err(|_| err())?;
            (self.variant_names.get(idx))
                .zip(self.variants.get(idx))
                .ok_or_else(err)?
        };
        Ok((index, name, &schema.ty))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        self.find(value.as_bytes())
            .ok_or_else(|| E::unknown_variant(value, self.variant_names))
    }

    fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        self.find(value).ok_or_else(|| match str::from_utf8(value) {
            Ok(value) => E::unknown_variant(value, self.variant_names),
            Err(_) => E::invalid_value(de::Unexpected::Bytes(value), &self),
        })
    }
}

impl<'a> VariantVisitor<'a> {
    fn find(&self, variant: &[u8]) -> Option<(u32, &'static str, &'a OwnedDataModelVariant)> {
        (self.variant_names.iter())
            .zip(self.variants)
            .enumerate()
            .find_map(|(index, (&name, schema))| {
                (name.as_bytes() == variant).then_some((index as u32, name, &schema.ty))
            })
    }
}

struct NewtypeVariantSeed<'a, S> {
    context: &'a Context<'a, Strategy>,
    schema: &'a OwnedNamedType,
    serializer: S,
    location: expecting::Variant<'static, expecting::data::Newtype<'a>>,
}

impl<'de, S: Serializer> DeserializeSeed<'de> for NewtypeVariantSeed<'_, S> {
    type Value = Result<S::Ok, S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.context
            .reserialize_ty(self.schema, deserializer, |inner| {
                self.serializer.serialize_newtype_variant(
                    self.location.enum_name,
                    self.location.variant_index,
                    self.location.variant_name,
                    inner,
                )
            })
    }
}
