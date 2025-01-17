use core::fmt;
use std::collections::HashMap;

use postcard_schema::schema::owned::{OwnedNamedType, OwnedNamedValue};
use serde::{
    de::{self, DeserializeSeed, Deserializer, Error as _, MapAccess, SeqAccess},
    ser::{self, Error as _, Serialize, SerializeStruct, Serializer},
};

use crate::reserialize::{
    expecting::{self, Unexpected},
    Context,
};

use super::Strategy;

pub struct Visitor<'a, S, Strategy, Reserializer> {
    pub context: &'a Context<'a, Strategy>,
    pub serializer: S,
    pub reserializer: Reserializer,
    pub fields: &'a [OwnedNamedValue],
    pub field_names: &'static [&'static str],
}

trait Reserializer<S: Serializer>: de::Expected {
    type SerializeFields: SerializeStruct<Ok = S::Ok, Error = S::Error>;

    fn reserialize_struct(
        &self,
        serializer: S,
        len: usize,
    ) -> Result<Self::SerializeFields, S::Error>;
}

struct SerializeStructVariant<T>(T);

impl<S: ser::SerializeStructVariant> SerializeStruct for SerializeStructVariant<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}

impl<S: Serializer> Reserializer<S> for expecting::Variant<'static, expecting::data::Struct<'_>> {
    type SerializeFields = SerializeStructVariant<S::SerializeStructVariant>;

    fn reserialize_struct(
        &self,
        serializer: S,
        len: usize,
    ) -> Result<Self::SerializeFields, S::Error> {
        serializer
            .serialize_struct_variant(self.enum_name, self.variant_index, self.variant_name, len)
            .map(SerializeStructVariant)
    }
}

impl<S: Serializer> Reserializer<S> for expecting::Struct<'static, expecting::data::Struct<'_>> {
    type SerializeFields = S::SerializeStruct;

    fn reserialize_struct(
        &self,
        serializer: S,
        len: usize,
    ) -> Result<Self::SerializeFields, S::Error> {
        serializer.serialize_struct(self.name, len)
    }
}

impl<'de, S, Reserializer> de::Visitor<'de> for Visitor<'_, S, Strategy, Reserializer>
where
    S: Serializer,
    Reserializer: self::Reserializer<S>,
{
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        de::Expected::fmt(&self.reserializer, formatter)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut serializer = match self
            .reserializer
            .reserialize_struct(self.serializer, self.field_names.len())
        {
            Ok(serializer) => serializer,
            Err(err) => return Ok(Err(err)),
        };
        let fields = (self.field_names.iter())
            .zip(self.fields)
            .map(|(&name, field)| (name, &field.ty));
        for (idx, (name, schema)) in fields.enumerate() {
            let seed = FieldSeed {
                context: self.context,
                serializer: &mut serializer,
                name,
                schema,
            };
            let res = seq.next_element_seed(seed)?.ok_or_else(|| {
                A::Error::missing_elements(idx, &self.reserializer, self.fields.len())
            })?;
            match res {
                Ok(()) => {}
                Err(err) => return Ok(Err(err)),
            }
        }
        Ok(serializer.end())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut serializer = match self
            .reserializer
            .reserialize_struct(self.serializer, self.field_names.len())
        {
            Ok(serializer) => serializer,
            Err(err) => return Ok(Err(err)),
        };

        let key = FieldVisitor {
            fields: self.fields,
            field_names: self.field_names,
        };
        let mut remaining_fields = self.field_names.iter().peekable();
        let mut out_of_order_fields = None;
        while let Some(field) = map.next_key_seed(&key)? {
            match field {
                Err(Ignored) => {
                    // This only works for self-describing formats, but it should only
                    // be self-describing formats that deserialize to ignored fields.
                    let de::IgnoredAny = map.next_value::<de::IgnoredAny>()?;
                }
                Ok((name, schema)) if remaining_fields.next_if_eq(&&name).is_some() => {
                    let res = map.next_value_seed(FieldSeed {
                        context: self.context,
                        serializer: &mut serializer,
                        name,
                        schema,
                    })?;
                    match res {
                        Ok(()) => {}
                        Err(err) => return Ok(Err(err)),
                    }
                }
                Ok((name, schema)) => {
                    // Fields were deserialized out-of-order. Serializers assume fields are
                    // serialized in-order, so buffer up the out of order fields then serialize
                    // them in order.
                    let out_of_order = out_of_order_fields.get_or_insert_with(|| {
                        OutOfOrderFields(HashMap::with_capacity(remaining_fields.len()))
                    });
                    let res: Result<(), serde_content::Error> = map.next_value_seed(FieldSeed {
                        context: self.context,
                        serializer: out_of_order,
                        name,
                        schema,
                    })?;
                    match res {
                        Ok(()) => {}
                        Err(err) => return Ok(Err(S::Error::custom(err))),
                    }
                }
            }
        }
        let mut out_of_order = out_of_order_fields
            .map(|OutOfOrderFields(out_of_order)| out_of_order)
            .unwrap_or(HashMap::new());
        for field in remaining_fields {
            match out_of_order.remove(field) {
                Some(value) => match serializer.serialize_field(field, &value) {
                    Ok(()) => {}
                    Err(err) => return Ok(Err(err)),
                },
                None => return Err(A::Error::missing_field(field)),
            }
        }
        for field in self.field_names {
            if out_of_order.contains_key(field) {
                return Err(A::Error::duplicate_field(field));
            }
        }
        Ok(serializer.end())
    }
}

struct FieldSeed<'a, S> {
    context: &'a Context<'a, Strategy>,
    serializer: &'a mut S,
    name: &'static str,
    schema: &'a OwnedNamedType,
}

impl<'de, S: SerializeStruct> DeserializeSeed<'de> for FieldSeed<'_, S> {
    type Value = Result<(), S::Error>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.context
            .reserialize_ty(self.schema, deserializer, |value| {
                self.serializer.serialize_field(self.name, value)
            })
    }
}

struct FieldVisitor<'a> {
    fields: &'a [OwnedNamedValue],
    field_names: &'static [&'static str],
}

struct Ignored;

impl<'a, 'de> DeserializeSeed<'de> for &FieldVisitor<'a> {
    type Value = Result<(&'static str, &'a OwnedNamedType), Ignored>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}

impl<'a> de::Visitor<'_> for &FieldVisitor<'a> {
    type Value = Result<(&'static str, &'a OwnedNamedType), Ignored>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "field identifier")
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok((usize::try_from(value).ok())
            .and_then(|idx| {
                let (&name, schema) = (self.field_names.get(idx)).zip(self.fields.get(idx))?;
                Some((name, &schema.ty))
            })
            .ok_or(Ignored))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(self.find(value.as_bytes()))
    }

    fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        Ok(self.find(value))
    }
}

impl<'a> FieldVisitor<'a> {
    fn find(&self, field: &[u8]) -> Result<(&'static str, &'a OwnedNamedType), Ignored> {
        self.field_names
            .iter()
            .zip(self.fields)
            .find_map(|(&name, schema)| (name.as_bytes() == field).then_some((name, &schema.ty)))
            .ok_or(Ignored)
    }
}

#[derive(Debug)]
struct OutOfOrderFields<'a>(HashMap<&'a str, serde_content::Value<'a>>);

impl<'a> SerializeStruct for OutOfOrderFields<'a> {
    type Ok = HashMap<&'a str, serde_content::Value<'a>>;
    type Error = serde_content::Error;

    fn serialize_field<T: ?Sized + serde::Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let value = value.serialize(serde_content::Serializer::new())?;
        debug_assert!(self.0.len() < self.0.capacity());
        self.0.insert(key, value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0)
    }
}
