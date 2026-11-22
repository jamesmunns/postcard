use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use postcard_core::de::{
    flavors::Slice, try_take_bool, try_take_bytes, try_take_char, try_take_discriminant,
    try_take_f32, try_take_f64, try_take_i128, try_take_i16, try_take_i32, try_take_i64,
    try_take_i8, try_take_isize, try_take_length, try_take_option_discrim, try_take_str,
    try_take_u128, try_take_u16, try_take_u32, try_take_u64, try_take_u8, try_take_usize,
    DecodeError, UnexpectedEnd,
};
use postcard_schema::schema::owned::{OwnedData, OwnedDataModelType};
use serde_json::{Map, Number, Value};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedEndOfData,
    ShouldSupportButDont,
    SchemaMismatch,
    Unsupported,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedEndOfData => write!(f, "reached end of data unexpectedly"),
            Error::ShouldSupportButDont => write!(
                f,
                "reached a limitation of using serde_json::Value as the decoding target"
            ),
            Error::SchemaMismatch => {
                write!(f, "reached data that was not correct for the given schema")
            }
            Error::Unsupported => write!(f, "reached unsupported item"),
        }
    }
}

impl std::error::Error for Error {}

trait GetExt {
    type Out;
    fn right(self) -> Result<Self::Out, Error>;
}

impl<T> GetExt for Option<T> {
    type Out = T;

    fn right(self) -> Result<Self::Out, Error> {
        self.ok_or(Error::SchemaMismatch)
    }
}

/// Map a postcard-core decode error onto this crate's [`Error`].
trait DecodeExt {
    type Out;
    fn dec(self) -> Result<Self::Out, Error>;
}

impl<T> DecodeExt for Result<T, DecodeError<UnexpectedEnd>> {
    type Out = T;

    fn dec(self) -> Result<Self::Out, Error> {
        self.map_err(|e| match e {
            DecodeError::Pop(UnexpectedEnd) => Error::UnexpectedEndOfData,
            _ => Error::SchemaMismatch,
        })
    }
}

pub fn from_slice_dyn(schema: &OwnedDataModelType, data: &[u8]) -> Result<Value, Error> {
    let mut f = Slice::new(data);
    deserialize(schema, &mut f)
}

fn deserialize(ty: &OwnedDataModelType, f: &mut Slice<'_>) -> Result<Value, Error> {
    let val = match ty {
        OwnedDataModelType::Bool => Value::Bool(try_take_bool(f).dec()?),
        OwnedDataModelType::I8 => Value::Number(Number::from(try_take_i8(f).dec()?)),
        OwnedDataModelType::U8 => Value::Number(Number::from(try_take_u8(f).dec()?)),
        OwnedDataModelType::I16 => Value::Number(Number::from(try_take_i16(f).dec()?)),
        OwnedDataModelType::I32 => Value::Number(Number::from(try_take_i32(f).dec()?)),
        OwnedDataModelType::I64 => Value::Number(Number::from(try_take_i64(f).dec()?)),
        OwnedDataModelType::I128 => {
            let val = try_take_i128(f).dec()?;
            let val = i64::try_from(val).map_err(|_| Error::ShouldSupportButDont)?;
            Value::Number(Number::from(val))
        }
        OwnedDataModelType::U16 => Value::Number(Number::from(try_take_u16(f).dec()?)),
        OwnedDataModelType::U32 => Value::Number(Number::from(try_take_u32(f).dec()?)),
        OwnedDataModelType::U64 => Value::Number(Number::from(try_take_u64(f).dec()?)),
        OwnedDataModelType::U128 => {
            let val = try_take_u128(f).dec()?;
            let val = u64::try_from(val).map_err(|_| Error::ShouldSupportButDont)?;
            Value::Number(Number::from(val))
        }
        OwnedDataModelType::Usize => Value::Number(Number::from(try_take_usize(f).dec()?)),
        OwnedDataModelType::Isize => Value::Number(Number::from(try_take_isize(f).dec()?)),
        OwnedDataModelType::F32 => {
            let val = try_take_f32(f).dec()?;
            Value::Number(Number::from_f64(val.into()).right()?)
        }
        OwnedDataModelType::F64 => {
            let val = try_take_f64(f).dec()?;
            Value::Number(Number::from_f64(val).right()?)
        }
        OwnedDataModelType::Char => Value::String(try_take_char(f).dec()?.to_string()),
        OwnedDataModelType::String { .. } => Value::String(try_take_str(f).dec()?.to_string()),
        OwnedDataModelType::ByteArray { .. } => {
            let bytes = try_take_bytes(f).dec()?;
            let vvec = bytes
                .iter()
                .map(|b| Value::Number(Number::from(*b)))
                .collect::<Vec<Value>>();
            Value::Array(vvec)
        }
        OwnedDataModelType::Option(inner) => {
            if try_take_option_discrim(f).dec()? {
                deserialize(inner, f)?
            } else {
                Value::Null
            }
        }
        OwnedDataModelType::Unit
        | OwnedDataModelType::Struct {
            name: _,
            data: OwnedData::Unit,
        } => {
            // TODO This is PROBABLY wrong, as Some(()) will be coalesced into the same
            // value as None. Fix this when we have our own Value
            Value::Null
        }
        OwnedDataModelType::Struct {
            name: _,
            data: OwnedData::Newtype(ty),
        } => deserialize(ty, f)?,
        OwnedDataModelType::Seq { element: ty, .. } => {
            let len = try_take_length(f).dec()?;
            let mut vec = vec![];
            for _ in 0..len {
                vec.push(deserialize(ty, f)?);
            }
            Value::Array(vec)
        }
        OwnedDataModelType::Tuple(tys)
        | OwnedDataModelType::Struct {
            name: _,
            data: OwnedData::Tuple(tys),
        } => {
            match &tys[..] {
                [] => {
                    // `serde_json` writes an empty tuple struct as `[]`, which is also what
                    // the encoder expects.
                    Value::Array(vec![])
                }
                [ty] => {
                    // Single item, NOT an array
                    deserialize(ty, f)?
                }
                multi => {
                    let mut vec = vec![];
                    for ty in multi.iter() {
                        vec.push(deserialize(ty, f)?);
                    }
                    Value::Array(vec)
                }
            }
        }
        OwnedDataModelType::Map { key, val, .. } => {
            // TODO: There's also a mismatch here because serde_json::Value requires
            // keys to be strings, when postcard doesn't.
            if !matches!(key.deref(), &OwnedDataModelType::String { max_len: _ }) {
                return Err(Error::ShouldSupportButDont);
            }

            let map_len = try_take_length(f).dec()?;
            let mut map = Map::new();

            for _ in 0..map_len {
                let s = try_take_str(f).dec()?;
                let v = deserialize(val, f)?;
                map.insert(s.to_string(), v);
            }

            Value::Object(map)
        }
        OwnedDataModelType::Struct {
            name: _,
            data: OwnedData::Struct(nvs),
        } => {
            let mut map = Map::new();
            for nv in nvs.iter() {
                let val = deserialize(&nv.ty, f)?;
                map.insert(nv.name.to_string(), val);
            }
            Value::Object(map)
        }
        OwnedDataModelType::Enum {
            name: _,
            variants: nvars,
        } => {
            let variant = try_take_discriminant(f).dec()?;
            let variant = usize::try_from(variant).map_err(|_| Error::SchemaMismatch)?;
            let schema = nvars.get(variant).right()?;
            match &schema.data {
                OwnedData::Unit => {
                    // Units become strings
                    Value::String(schema.name.to_string())
                }
                OwnedData::Newtype(ty) => {
                    // everything else becomes an object with one field
                    let val = deserialize(ty, f)?;
                    let mut map = Map::new();
                    map.insert(schema.name.to_owned().to_string(), val);
                    Value::Object(map)
                }
                OwnedData::Tuple(vec) => {
                    // everything else becomes an object with one field
                    let val = deserialize(&OwnedDataModelType::Tuple(vec.clone()), f)?;
                    let mut map = Map::new();
                    map.insert(schema.name.to_owned().to_string(), val);
                    Value::Object(map)
                }
                OwnedData::Struct(vec) => {
                    // everything else becomes an object with one field
                    let val = deserialize(
                        &OwnedDataModelType::Struct {
                            name: schema.name.clone(),
                            data: OwnedData::Struct(vec.clone()),
                        },
                        f,
                    )?;
                    let mut map = Map::new();
                    map.insert(schema.name.to_owned().to_string(), val);
                    Value::Object(map)
                }
            }
        }
        OwnedDataModelType::Schema => return Err(Error::Unsupported),
    };
    Ok(val)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use postcard_schema::Schema;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::Error;
    use crate::{from_slice_dyn, to_stdvec_dyn};

    #[derive(Serialize, Deserialize, Schema)]
    struct Struct1 {
        pub x: bool,
        pub y: u16,
        pub z: f64,
    }

    #[derive(Serialize, Deserialize, Schema)]
    enum Enum1 {
        Alpha,
        Beta(u8),
        Gamma(Vec<u8>),
        Delta(Struct1),
        Epsilon(u8, bool),
        Zeta { a: u8, b: bool },
    }

    #[test]
    fn smoke() {
        let bye = serde_json::to_value(Enum1::Alpha).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![0], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                "Alpha"
            }
        );

        let bye = serde_json::to_value(Enum1::Beta(4)).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![1, 4], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                {"Beta": 4}
            }
        );

        let bye = serde_json::to_value(Enum1::Gamma(vec![1, 2, 3])).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![2, 3, 1, 2, 3], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                {"Gamma": [1, 2, 3]}
            }
        );

        let bye = serde_json::to_value(Enum1::Delta(Struct1 {
            x: false,
            y: 1000,
            z: 4.0,
        }))
        .unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![3, 0, 232, 7, 0, 0, 0, 0, 0, 0, 16, 64], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                {"Delta": {
                    "x": false,
                    "y": 1000,
                    "z": 4.0
                }}
            }
        );

        let bye = serde_json::to_value(Enum1::Epsilon(8, false)).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![4, 8, 0], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                {"Epsilon": [8, false]}
            }
        );

        let bye = serde_json::to_value(Enum1::Zeta { a: 9, b: true }).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![5, 9, 1], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                {"Zeta": {"a": 9, "b": true}}
            }
        );
    }

    #[test]
    fn chars() {
        let de = from_slice_dyn(&char::SCHEMA.into(), &[2, 0xC3, 0xA9]).unwrap();
        assert_eq!(de, json!("é"));

        // Two chars, and a lone continuation byte are both rejected
        for bad in [&[2, b'a', b'b'][..], &[1, 0x80]] {
            let res = from_slice_dyn(&char::SCHEMA.into(), bad);
            assert_eq!(res, Err(Error::SchemaMismatch));
        }
    }

    #[test]
    fn discriminant_is_u32() {
        // Variant 1 (`Beta(4)`) with a 6-byte discriminant: longer than a u32
        // varint allows
        let data = [0x81, 0x80, 0x80, 0x80, 0x80, 0x00, 4];
        let res = from_slice_dyn(&Enum1::SCHEMA.into(), &data);
        assert_eq!(res, Err(Error::SchemaMismatch));
    }

    #[test]
    fn truncated() {
        let res = from_slice_dyn(&Struct1::SCHEMA.into(), &[0, 232]);
        assert_eq!(res, Err(Error::UnexpectedEndOfData));
    }

    #[test]
    fn empty_tuple_struct() {
        #[derive(Serialize, Deserialize, Schema)]
        struct Empty();

        let val = serde_json::to_value(Empty()).unwrap();
        let t = to_stdvec_dyn(&Empty::SCHEMA.into(), &val).unwrap();
        assert!(t.is_empty());
        let de = from_slice_dyn(&Empty::SCHEMA.into(), &t).unwrap();
        assert_eq!(de, val);
    }

    #[test]
    fn maps() {
        type Map1 = HashMap<String, i64>;
        let mut map: Map1 = HashMap::new();
        map.insert("bib".to_string(), 10);
        map.insert("bim".to_string(), 20);
        map.insert("bap".to_string(), 30);
        let bym = serde_json::to_value(map).unwrap();
        let t = to_stdvec_dyn(&Map1::SCHEMA.into(), &bym).unwrap();
        let de = from_slice_dyn(&Map1::SCHEMA.into(), &t).unwrap();
        assert_eq!(de, bym);

        let de = from_slice_dyn(&Map1::SCHEMA.into(), &[1, 3, b'b', b'i', b'b', 3]).unwrap();
        assert_eq!(de, json!({"bib": -2}));
    }

    #[test]
    fn schema_is_unsupported() {
        use postcard_schema::schema::owned::OwnedDataModelType;

        let res = from_slice_dyn(&OwnedDataModelType::Schema, &[]);
        assert_eq!(res, Err(Error::Unsupported));
        let res = to_stdvec_dyn(&OwnedDataModelType::Schema, &json!(null));
        assert_eq!(res, Err(crate::ser::Error::Unsupported));
    }
}
