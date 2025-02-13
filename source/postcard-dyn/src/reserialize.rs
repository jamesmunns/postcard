//! Dynamically reserialize [`postcard`]-encoded values from [`Deserializer`]s into [`Serializer`]s.
//!
//! This module implements transformations between postcard-encoded data and other serialized forms
//! based on [dynamic schemas](postcard_schema::schema::owned). For example, this could be used to
//! transform postcard-encoded data to JSON or another human-readable format, or to transform JSON
//! to postcard.
//!
//! # Limitations
//!
//! Several [`Deserializer`] and [`Serializer`] methods require `&'static` parameters, namely:
//! - [`Serializer`] methods for serializing structs and enums require `&'static str`s for the
//!   names of structs, fields, enums, and variants.
//! - [`Deserializer`] methods for deserializing structs and enums require the same `&'static str`s
//!   as the corresponding serialize methods, and moreover `&'static [&'static str]`s for the names
//!   of fields in structs.
//!
//! Since these transformations work with dynamic schemas that contain [`String`]s instead of
//! `&'static str`s, lossless reserialization is possible only with other compromises.
//!
//! In particular, reserialization can be either:
//! - Lossless with implementation compromises: see [`lossless`]
//! - Lossy with regards to structs and enums: see [`lossy`]

use core::{cell::Cell, fmt, marker::PhantomData, slice};

use postcard_schema::schema::owned::{OwnedData, OwnedDataModelType};
use serde::{de, ser::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::Error;

pub mod lossless;
pub mod lossy;

mod expecting;
mod strategy;
use strategy::Strategy;

mod map;
mod option;
mod seq;
mod tuple;

struct Context<'a, Strategy> {
    strategy: &'a Strategy,
}

struct Reserialize<F: ReserializeFn> {
    f: Cell<Option<F>>,
    deserializer_error: Cell<Option<F::DeserializeError>>,
}

trait ReserializeFn {
    type DeserializeError: de::Error;

    fn reserialize<S: Serializer>(
        self,
        serializer: S,
    ) -> Result<S::Ok, Error<Self::DeserializeError, S::Error>>;
}

impl<F: ReserializeFn> serde::Serialize for Reserialize<F> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let f = self.f.take().unwrap();
        match f.reserialize(serializer) {
            Ok(out) => Ok(out),
            Err(Error::Serialize(err)) => Err(err),
            Err(Error::Deserialize(err)) => {
                let res = Err(S::Error::custom(format_args!("{err}")));
                self.deserializer_error.set(Some(err));
                res
            }
        }
    }
}

impl<Strategy: strategy::Strategy> Context<'_, Strategy> {
    fn reserialize<F: ReserializeFn, T>(
        &self,
        reserialize: F,
        f: impl FnOnce(&Reserialize<F>) -> T,
    ) -> Result<T, F::DeserializeError> {
        let reserialize = Reserialize {
            f: Cell::new(Some(reserialize)),
            deserializer_error: Cell::new(None),
        };
        let res = f(&reserialize);
        match reserialize.deserializer_error.take() {
            Some(err) => Err(err),
            None => Ok(res),
        }
    }

    fn reserialize_ty<'de, D: Deserializer<'de>, T>(
        &self,
        schema: &OwnedDataModelType,
        deserializer: D,
        f: impl FnOnce(&Reserialize<ReserializeTy<'_, 'de, D, Strategy>>) -> T,
    ) -> Result<T, D::Error> {
        self.reserialize(
            ReserializeTy {
                context: self,
                deserializer,
                schema,
                de: PhantomData,
            },
            f,
        )
    }
}

struct ReserializeTy<'a, 'de, D, Strategy> {
    context: &'a Context<'a, Strategy>,
    deserializer: D,
    schema: &'a OwnedDataModelType,
    de: PhantomData<&'de ()>,
}

impl<'de, D, Strategy> ReserializeFn for ReserializeTy<'_, 'de, D, Strategy>
where
    D: Deserializer<'de>,
    Strategy: strategy::Strategy,
{
    type DeserializeError = D::Error;

    fn reserialize<S: Serializer>(
        self,
        serializer: S,
    ) -> Result<S::Ok, Error<Self::DeserializeError, S::Error>> {
        fn deserialize<'de, T, D, SerializerError>(
            deserializer: D,
        ) -> Result<T, Error<D::Error, SerializerError>>
        where
            T: Deserialize<'de>,
            D: Deserializer<'de>,
        {
            T::deserialize(deserializer).map_err(Error::Deserialize)
        }
        let (context, deserializer, schema) = (self.context, self.deserializer, self.schema);
        match schema {
            OwnedDataModelType::Schema => OwnedDataModelType::deserialize(deserializer)
                .map_err(Error::Deserialize)?
                .serialize(serializer),
            OwnedDataModelType::Unit => serializer.serialize_unit(),
            OwnedDataModelType::Bool => serializer.serialize_bool(deserialize(deserializer)?),
            OwnedDataModelType::U8 => serializer.serialize_u8(deserialize(deserializer)?),
            OwnedDataModelType::U16 => serializer.serialize_u16(deserialize(deserializer)?),
            OwnedDataModelType::U32 => serializer.serialize_u32(deserialize(deserializer)?),
            OwnedDataModelType::U64 => serializer.serialize_u64(deserialize(deserializer)?),
            OwnedDataModelType::U128 => serializer.serialize_u128(deserialize(deserializer)?),
            OwnedDataModelType::I8 => serializer.serialize_i8(deserialize(deserializer)?),
            OwnedDataModelType::I16 => serializer.serialize_i16(deserialize(deserializer)?),
            OwnedDataModelType::I32 => serializer.serialize_i32(deserialize(deserializer)?),
            OwnedDataModelType::I64 => serializer.serialize_i64(deserialize(deserializer)?),
            OwnedDataModelType::I128 => serializer.serialize_i128(deserialize(deserializer)?),
            OwnedDataModelType::Usize => {
                deserialize::<usize, _, _>(deserializer)?.serialize(serializer)
            }
            OwnedDataModelType::Isize => {
                deserialize::<isize, _, _>(deserializer)?.serialize(serializer)
            }
            OwnedDataModelType::F32 => serializer.serialize_f32(deserialize(deserializer)?),
            OwnedDataModelType::F64 => serializer.serialize_f64(deserialize(deserializer)?),
            OwnedDataModelType::Char => serializer.serialize_char(deserialize(deserializer)?),
            OwnedDataModelType::String => serializer.serialize_str(deserialize(deserializer)?),
            OwnedDataModelType::ByteArray => serializer.serialize_bytes(deserialize(deserializer)?),
            OwnedDataModelType::Option(inner) => deserializer
                .deserialize_option(option::Visitor {
                    context,
                    serializer,
                    schema: inner,
                })
                .map_err(Error::Deserialize)?,
            OwnedDataModelType::Map { key, val } => deserializer
                .deserialize_map(map::Visitor {
                    context,
                    serializer,
                    key,
                    val,
                })
                .map_err(Error::Deserialize)?,
            OwnedDataModelType::Seq(element) => deserializer
                .deserialize_seq(seq::Visitor {
                    context,
                    serializer,
                    schemas: slice::from_ref(element),
                })
                .map_err(Error::Deserialize)?,
            OwnedDataModelType::Tuple(elements) => deserializer
                .deserialize_tuple(
                    elements.len(),
                    tuple::Visitor {
                        context,
                        serializer,
                        fields: elements,
                        reserializer: expecting::Tuple,
                    },
                )
                .map_err(Error::Deserialize)?,
            OwnedDataModelType::Struct { name, data } => match data {
                OwnedData::Unit => {
                    Strategy::reserialize_unit_struct(context, deserializer, serializer, name)
                        .map_err(Error::Deserialize)?
                }
                OwnedData::Newtype(inner) => Strategy::reserialize_newtype_struct(
                    context,
                    deserializer,
                    serializer,
                    expecting::Struct {
                        name,
                        data: expecting::data::Newtype { schema: inner },
                    },
                )
                .map_err(Error::Deserialize)?,
                OwnedData::Tuple(fields) => Strategy::reserialize_tuple_struct(
                    context,
                    deserializer,
                    serializer,
                    expecting::Struct {
                        name,
                        data: expecting::data::Tuple { elements: fields },
                    },
                )
                .map_err(Error::Deserialize)?,
                OwnedData::Struct(fields) => Strategy::reserialize_struct(
                    context,
                    deserializer,
                    serializer,
                    expecting::Struct {
                        name,
                        data: expecting::data::Struct { fields },
                    },
                )
                .map_err(Error::Deserialize)?,
            },
            OwnedDataModelType::Enum { name, variants } => Strategy::reserialize_enum(
                context,
                deserializer,
                serializer,
                expecting::Enum { name, variants },
            )
            .map_err(Error::Deserialize)?,
        }
        .map_err(Error::Serialize)
    }
}

struct Expected<'a>(fmt::Arguments<'a>);

impl de::Expected for Expected<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;
    use postcard::ser_flavors::Flavor;
    use postcard_schema::Schema;
    use serde::de::DeserializeOwned;
    use serde_json::json;

    use super::*;

    #[derive(Serialize, Deserialize, Schema, PartialEq, Debug)]
    enum Enum {
        Struct { a: u8, b: u8 },
        Tuple(bool, u8),
        Newtype(u32),
        Unit,
    }

    #[derive(Serialize, Deserialize, Schema, PartialEq, Debug)]
    struct Struct {
        a: Option<u8>,
        b: Enum,
        c: u8,
    }

    fn postcard_to_json<T: Schema>(postcard: &[u8]) -> serde_json::Value {
        let schema = T::SCHEMA.into();
        let leaky = lossless::reserialize_leaky(
            &schema,
            &mut postcard::Deserializer::from_bytes(postcard),
            serde_json::value::Serializer,
        )
        .unwrap();
        let lossy = lossy::reserialize_with_structs_and_enums_as_maps(
            &schema,
            &mut postcard::Deserializer::from_bytes(postcard),
            serde_json::value::Serializer,
        )
        .unwrap();
        assert_eq!(leaky, lossy);
        leaky
    }

    fn json_to_postcard<T: Schema>(json: &serde_json::Value) -> Vec<u8> {
        let mut serializer = postcard::Serializer {
            output: postcard::ser_flavors::AllocVec::new(),
        };
        lossless::reserialize_leaky(&T::SCHEMA.into(), json, &mut serializer).unwrap();
        serializer.output.finalize().unwrap()
    }

    fn test_postcard_to_json_and_back<T>(value: T)
    where
        T: Schema + Serialize + DeserializeOwned + Debug + PartialEq,
    {
        let postcard_bytes = postcard::to_allocvec(&value).unwrap();
        let json = postcard_to_json::<T>(&postcard_bytes);
        assert_eq!(json, serde_json::to_value(&value).unwrap());
        assert_eq!(T::deserialize(&json).unwrap(), value);

        let roundtripped_postcard_bytes = json_to_postcard::<T>(&json);
        assert_eq!(roundtripped_postcard_bytes, postcard_bytes);
        assert_eq!(
            postcard::from_bytes::<T>(&roundtripped_postcard_bytes).unwrap(),
            value
        );
    }

    fn test_json_to_postcard<T>(json: serde_json::Value)
    where
        T: Schema + Serialize + DeserializeOwned + Debug + PartialEq,
    {
        let postcard_bytes = json_to_postcard::<T>(&json);
        let from_json = T::deserialize(&json).unwrap();
        let from_postcard_bytes = postcard::from_bytes::<T>(&postcard_bytes).unwrap();
        assert_eq!(from_postcard_bytes, from_json);
    }

    fn test_json_to_postcard_and_back<T>(json: serde_json::Value)
    where
        T: Schema + Serialize + DeserializeOwned + Debug + PartialEq,
    {
        let postcard_bytes = json_to_postcard::<T>(&json);

        let from_json = T::deserialize(&json).unwrap();
        let from_postcard_bytes = postcard::from_bytes::<T>(&postcard_bytes).unwrap();
        assert_eq!(from_postcard_bytes, from_json);

        let json_roundtripped = postcard_to_json::<T>(&postcard_bytes);
        assert_eq!(json_roundtripped, json);

        let from_json_roundtripped = T::deserialize(&json_roundtripped).unwrap();
        assert_eq!(from_json_roundtripped, from_json);
    }

    #[test]
    fn json() {
        use test_postcard_to_json_and_back as test;
        test(Enum::Struct { a: 5, b: 10 });
        test(Enum::Tuple(false, 15));
        test(Enum::Newtype(20));
        test(Enum::Unit);
        test(Struct {
            a: Some(5),
            b: Enum::Struct { a: 10, b: 100 },
            c: 7,
        });
    }

    #[test]
    /// Make sure reserialization handles out-of-order struct fields correctly.
    /// Serializers like postcard rely on struct fields being serialized in order.
    fn out_of_order_fields() {
        use test_json_to_postcard_and_back as test;
        test::<Enum>(json!({"Struct": {"b": 10, "a": 5}}));
        test::<Enum>(json!({"Struct": {"a": 5, "b": 0}}));

        let nested = json!({"Struct": {"b": 50, "a": 100}});
        test::<Struct>(json!({"a": 5, "b": nested, "c": 10}));
        test::<Struct>(json!({"b": nested, "a": 5, "c": 10}));
        test::<Struct>(json!({"b": nested, "c": 10, "a": 5}));
        test::<Struct>(json!({"a": 5, "c": 10, "b": nested}));
        test::<Struct>(json!({"c": 10, "a": 5, "b": nested}));
    }

    #[test]
    fn extra_fields() {
        use test_json_to_postcard as test;
        test::<Enum>(json!({"Struct": {"a": 5, "b": 0, "UNUSED": 10}}));
        test::<Struct>(json!({"a": 5, "xyz": "wat", "b": {"Newtype": 32}, "c": 10}));
    }

    #[test]
    #[should_panic = "missing field `b`"]
    fn missing_fields() {
        test_json_to_postcard::<Enum>(json!({"Struct": {"a": 5}}));
    }

    #[test]
    #[should_panic = "invalid length 1, expected tuple variant Enum::Tuple with 2 elements"]
    fn missing_tuple_fields() {
        test_json_to_postcard::<Enum>(json!({"Tuple": [false]}));
    }

    #[test]
    /// Make sure both deserializer and serializer errors are bubbled up
    fn errors() {
        use postcard_schema::Schema;

        assert!(matches!(
            dbg!(lossless::reserialize_leaky(
                &u8::SCHEMA.into(),
                &mut postcard::Deserializer::from_bytes(&[]),
                serde_json::value::Serializer
            )),
            Err(Error::Deserialize(
                postcard::Error::DeserializeUnexpectedEnd
            ))
        ));
        assert!(matches!(
            dbg!(lossless::reserialize_leaky(
                &u8::SCHEMA.into(),
                &mut postcard::Deserializer::from_bytes(&[5]),
                &mut serde_json::Serializer::new(std::io::Cursor::new([].as_mut_slice()))
            )),
            Err(Error::Serialize(_))
        ));
    }
}
