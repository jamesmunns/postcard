use std::fmt::{Debug, Display};

use postcard_core::de::{
    DecodeError, Flavor, UnexpectedEnd, flavors::Slice, try_take_bool, try_take_bytes,
    try_take_char, try_take_discriminant, try_take_f32, try_take_f64, try_take_i8, try_take_i16,
    try_take_i32, try_take_i64, try_take_i128, try_take_length, try_take_option_discrim,
    try_take_str, try_take_u8, try_take_u16, try_take_u32, try_take_u64, try_take_u128,
};
use postcard_schema::schema::{OwnedData, OwnedDataModelType};
use serde_json::{Map, Number, Value};

/// The error returned when decoding a message.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    UnexpectedEndOfData,
    ShouldSupportButDont,
    SchemaMismatch,
    /// A limit of the [`DeserializeConfig`] was exceeded.
    LimitExceeded,
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
            Error::LimitExceeded => write!(f, "exceeded a configured limit"),
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

/// The limits used by [`from_slice_dyn_with_config`].
///
/// To change a limit, start from the default:
///
/// ```rust
/// use postcard_dyn::DeserializeConfig;
///
/// let mut config = DeserializeConfig::default();
/// config.max_zsts = 1024;
/// ```
// TODO: Limit the total number of `Value`s? One byte of input can be a whole object, and
// one byte of a `ByteArray` is a `Value`.
// TODO: Check lengths against the schema's `max_len`?
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DeserializeConfig {
    /// The maximum number of sequence elements that are encoded as zero bytes, such as
    /// the elements of a `Vec<()>`. Defaults to 128.
    ///
    /// This is the total for the whole message, as a message can contain many sequences.
    /// Zero-sized values outside of sequences are not counted.
    pub max_zsts: usize,

    /// The maximum length of the input in bytes, including any bytes after the message.
    /// Defaults to 1 MiB.
    pub max_input_len: usize,

    /// The maximum depth of nested types in the schema. Defaults to 128.
    pub max_depth: usize,
}

impl Default for DeserializeConfig {
    fn default() -> Self {
        Self {
            max_zsts: 128,
            max_input_len: 1024 * 1024,
            max_depth: 128,
        }
    }
}

/// Decode `data` as a message with the given schema with the default
/// [`DeserializeConfig`].
///
/// Bytes after the message are ignored.
pub fn from_slice_dyn(schema: &OwnedDataModelType, data: &[u8]) -> Result<Value, Error> {
    from_slice_dyn_with_config(schema, data, &DeserializeConfig::default())
}

/// Decode `data` as a message with the given schema.
///
/// Bytes after the message are ignored.
pub fn from_slice_dyn_with_config(
    schema: &OwnedDataModelType,
    data: &[u8],
    config: &DeserializeConfig,
) -> Result<Value, Error> {
    if data.len() > config.max_input_len {
        return Err(Error::LimitExceeded);
    }
    let mut f = Slice::new(data);
    let mut budget = Budget {
        zsts: Balance(config.max_zsts),
        depth: Balance(config.max_depth),
    };
    deserialize(schema, &mut f, &mut budget)
}

/// What is left of the limits in a [`DeserializeConfig`].
struct Budget {
    zsts: Balance,
    depth: Balance,
}

/// A helper type that counts down to zero, and returns an error when we do.
struct Balance(usize);

impl Balance {
    /// Take one from a limit.
    fn spend(&mut self) -> Result<(), Error> {
        self.0 = self.0.checked_sub(1).ok_or(Error::LimitExceeded)?;
        Ok(())
    }
}

fn deserialize(
    ty: &OwnedDataModelType,
    f: &mut Slice<'_>,
    budget: &mut Budget,
) -> Result<Value, Error> {
    budget.depth.spend()?;
    let val = deserialize_inner(ty, f, budget)?;
    budget.depth.0 += 1;
    Ok(val)
}

fn deserialize_inner(
    ty: &OwnedDataModelType,
    f: &mut Slice<'_>,
    budget: &mut Budget,
) -> Result<Value, Error> {
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
                deserialize(inner, f, budget)?
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
        } => deserialize(ty, f, budget)?,
        OwnedDataModelType::Seq { element: ty, .. } => {
            let len = try_take_length(f).dec()?;
            let mut vec = vec![];
            for _ in 0..len {
                let before = f.size_hint();
                let val = deserialize(ty, f, budget)?;
                // Other elements are limited by the length of the input.
                if f.size_hint() == before {
                    budget.zsts.spend()?;
                }
                vec.push(val);
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
                    deserialize(ty, f, budget)?
                }
                multi => {
                    let mut vec = vec![];
                    for ty in multi.iter() {
                        vec.push(deserialize(ty, f, budget)?);
                    }
                    Value::Array(vec)
                }
            }
        }
        OwnedDataModelType::Map { key, val, .. } => {
            let map_len = try_take_length(f).dec()?;
            let mut map = Map::new();

            for _ in 0..map_len {
                let k = de_map_key(key, f, budget)?;
                let v = deserialize(val, f, budget)?;
                map.insert(k, v);
            }

            Value::Object(map)
        }
        OwnedDataModelType::Struct {
            name: _,
            data: OwnedData::Struct(nvs),
        } => {
            let mut map = Map::new();
            for nv in nvs.iter() {
                let val = deserialize(&nv.ty, f, budget)?;
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
                    let val = deserialize(ty, f, budget)?;
                    let mut map = Map::new();
                    map.insert(schema.name.to_owned().to_string(), val);
                    Value::Object(map)
                }
                // TODO: This and the struct variant clone their schema for each value.
                OwnedData::Tuple(vec) => {
                    // everything else becomes an object with one field
                    let val =
                        deserialize_inner(&OwnedDataModelType::Tuple(vec.clone()), f, budget)?;
                    let mut map = Map::new();
                    map.insert(schema.name.to_owned().to_string(), val);
                    Value::Object(map)
                }
                OwnedData::Struct(vec) => {
                    // everything else becomes an object with one field
                    let val = deserialize_inner(
                        &OwnedDataModelType::Struct {
                            name: schema.name.clone(),
                            data: OwnedData::Struct(vec.clone()),
                        },
                        f,
                        budget,
                    )?;
                    let mut map = Map::new();
                    map.insert(schema.name.to_owned().to_string(), val);
                    Value::Object(map)
                }
            }
        }
    };
    Ok(val)
}

/// Deserialize a single map key.
///
/// `serde_json::Value` requires map keys to be strings, when postcard doesn't.
/// `serde_json` handles that by encoding primitive keys as their string
/// representation, so we do the same. Key types that `serde_json` also can't
/// represent as a string are still rejected.
fn de_map_key(
    ty: &OwnedDataModelType,
    f: &mut Slice<'_>,
    budget: &mut Budget,
) -> Result<String, Error> {
    match ty {
        OwnedDataModelType::Bool
        | OwnedDataModelType::I8
        | OwnedDataModelType::I16
        | OwnedDataModelType::I32
        | OwnedDataModelType::I64
        | OwnedDataModelType::I128
        | OwnedDataModelType::U8
        | OwnedDataModelType::U16
        | OwnedDataModelType::U32
        | OwnedDataModelType::U64
        | OwnedDataModelType::U128
        | OwnedDataModelType::String { .. }
        | OwnedDataModelType::Enum { .. } => {
            let key = match deserialize(ty, f, budget)? {
                // Unit variants of an enum come back as their name, anything
                // else is an object, which can't be a key.
                Value::String(s) => s,
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                _ => return Err(Error::ShouldSupportButDont),
            };
            Ok(key)
        }
        // TODO: `serde_json` also stores `char` and float keys as strings, but
        // round tripping a float through a decimal string is not something we
        // want to do silently.
        _ => Err(Error::ShouldSupportButDont),
    }
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};

    use postcard_schema::{Schema, schema::OwnedDataModelType};
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    use super::Error;
    use super::{DeserializeConfig, from_slice_dyn_with_config};
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

    #[derive(Serialize, Deserialize, Schema, PartialEq, Eq, PartialOrd, Ord)]
    enum Key1 {
        Alpha,
        Beta,
    }

    /// Like [`Key1`], but declared in an order the variant names don't sort in.
    #[derive(Serialize, Deserialize, Schema, PartialEq, Eq, PartialOrd, Ord)]
    enum Key2 {
        Zeta,
        Alpha,
    }

    /// Round trip a map through postcard, into a `Value`, and back into postcard,
    /// checking the `Value` along the way.
    fn map_round_trip<T: Serialize + Schema>(map: &T, expect: Value) {
        let schema = OwnedDataModelType::from(T::SCHEMA);
        let bytes = postcard::to_vec(map).unwrap();

        let de = from_slice_dyn(&schema, &bytes).unwrap();
        assert_eq!(de, expect);
        assert_eq!(de, serde_json::to_value(map).unwrap());

        let ser = to_stdvec_dyn(&schema, &de).unwrap();
        assert_eq!(ser, bytes);
    }

    #[test]
    fn non_string_map_keys() {
        // String keys, which have always worked
        map_round_trip(
            &BTreeMap::from([("bib".to_string(), 10u8), ("bim".to_string(), 20)]),
            json! {{"bib": 10, "bim": 20}},
        );

        // Anything serde_json can store as a string key, we can too
        map_round_trip(
            &BTreeMap::from([(false, 10u8), (true, 20)]),
            json! {{"false": 10, "true": 20}},
        );
        map_round_trip(
            &BTreeMap::from([(1u32, 10u8), (2, 20)]),
            json! {{"1": 10, "2": 20}},
        );
        map_round_trip(
            &BTreeMap::from([(-1i64, 10u8), (5, 20)]),
            json! {{"-1": 10, "5": 20}},
        );
        map_round_trip(
            &BTreeMap::from([(Key1::Alpha, 10u8), (Key1::Beta, 20)]),
            json! {{"Alpha": 10, "Beta": 20}},
        );

        // Keys only sort the same as their string form when they are strings,
        // so every other key type needs its entries put back in key order
        // before they are written out again.

        // Unsigned, digit widths that sort the other way around as strings
        map_round_trip(
            &BTreeMap::from([(2u32, 20u8), (10, 100)]),
            json! {{"2": 20, "10": 100}},
        );

        // Negative, same digit width, which sorts backwards as strings
        map_round_trip(
            &BTreeMap::from([(-1i8, 1u8), (-2, 2)]),
            json! {{"-1": 1, "-2": 2}},
        );

        // Mixed sign
        map_round_trip(
            &BTreeMap::from([(-2i16, 1u8), (-1, 2), (3, 3), (10, 4)]),
            json! {{"-2": 1, "-1": 2, "3": 3, "10": 4}},
        );

        // Unit variants sort by declaration order, not by name
        map_round_trip(
            &BTreeMap::from([(Key2::Zeta, 10u8), (Key2::Alpha, 20)]),
            json! {{"Zeta": 10, "Alpha": 20}},
        );

        // Nested maps get the same treatment
        map_round_trip(
            &BTreeMap::from([
                (2u32, BTreeMap::from([(2u32, 1u8), (10, 2)])),
                (10, BTreeMap::new()),
            ]),
            json! {{"2": {"2": 1, "10": 2}, "10": {}}},
        );

        // Keys serde_json can't store as a string are still rejected, rather
        // than silently mangled
        type Map1 = BTreeMap<(u8, u8), u8>;
        let map = Map1::from([((1, 2), 10)]);
        let bytes = postcard::to_vec(&map).unwrap();
        assert_eq!(
            from_slice_dyn(&Map1::SCHEMA.into(), &bytes),
            Err(Error::ShouldSupportButDont)
        );
        assert_eq!(
            to_stdvec_dyn(&Map1::SCHEMA.into(), &json! {{"[1,2]": 10}}),
            Err(crate::ser::Error::ShouldSupportButDont)
        );
    }

    #[test]
    fn max_zsts() {
        #[derive(Schema)]
        #[allow(dead_code)]
        struct Empty {}

        fn decode<T: Schema + ?Sized>(data: &[u8]) -> Result<usize, Error> {
            let value = from_slice_dyn(&T::SCHEMA.into(), data)?;
            Ok(value.as_array().unwrap().len())
        }

        const AT: [u8; 2] = [0x80, 0x01];
        const OVER: [u8; 2] = [0x81, 0x01];
        assert_eq!(decode::<Vec<()>>(&AT), Ok(128));
        assert_eq!(decode::<Vec<()>>(&OVER), Err(Error::LimitExceeded));
        assert_eq!(decode::<Vec<((), Empty)>>(&AT), Ok(128));
        assert_eq!(decode::<Vec<((), Empty)>>(&OVER), Err(Error::LimitExceeded));
        let huge = [0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
        assert_eq!(decode::<Vec<()>>(&huge), Err(Error::LimitExceeded));

        // The limit is for the whole message.
        let nested = [2, 64, 64];
        assert_eq!(decode::<Vec<Vec<()>>>(&nested), Ok(2));
        let nested = [2, 64, 65];
        assert_eq!(decode::<Vec<Vec<()>>>(&nested), Err(Error::LimitExceeded));

        // Elements that take bytes aren't counted, even if they contain a `()`.
        let mut data = vec![0xE8, 0x07];
        data.extend([0; 1000]);
        assert_eq!(decode::<Vec<u8>>(&data), Ok(1000));
        assert_eq!(decode::<Vec<((), u8)>>(&data), Ok(1000));

        let schema = <Vec<()>>::SCHEMA.into();
        let config = DeserializeConfig {
            max_zsts: 129,
            ..Default::default()
        };
        assert!(from_slice_dyn_with_config(&schema, &OVER, &config).is_ok());
        let config = DeserializeConfig {
            max_zsts: 0,
            ..Default::default()
        };
        let res = from_slice_dyn_with_config(&schema, &[1], &config);
        assert_eq!(res, Err(Error::LimitExceeded));
        assert_eq!(
            from_slice_dyn_with_config(&schema, &[0], &config),
            Ok(json!([]))
        );
    }

    #[test]
    fn max_input_len() {
        let schema = u8::SCHEMA.into();
        let mut data = vec![0; 1024 * 1024];
        assert_eq!(from_slice_dyn(&schema, &data), Ok(json!(0)));
        data.push(0);
        assert_eq!(from_slice_dyn(&schema, &data), Err(Error::LimitExceeded));

        let config = DeserializeConfig {
            max_input_len: data.len(),
            ..Default::default()
        };
        assert_eq!(
            from_slice_dyn_with_config(&schema, &data, &config),
            Ok(json!(0))
        );
    }

    #[test]
    fn max_depth() {
        use crate::{SerializeConfig, to_stdvec_dyn_with_config};
        use postcard_schema::schema::{
            OwnedData, OwnedDataModelType, OwnedNamedField, OwnedVariant,
        };

        /// A `u8` in newtypes or enum variants, with this many types in total.
        fn nested(
            depth: usize,
            variant: Option<fn(OwnedDataModelType) -> OwnedData>,
        ) -> OwnedDataModelType {
            let mut ty = OwnedDataModelType::U8;
            for _ in 1..depth {
                let name = "N".into();
                ty = match variant {
                    None => OwnedDataModelType::Struct {
                        name,
                        data: OwnedData::Newtype(Box::new(ty)),
                    },
                    Some(data) => OwnedDataModelType::Enum {
                        name,
                        variants: Box::new([OwnedVariant {
                            name: "V".into(),
                            data: data(ty),
                        }]),
                    },
                };
            }
            ty
        }

        #[allow(clippy::type_complexity)]
        let variants: [Option<fn(OwnedDataModelType) -> OwnedData>; 5] = [
            None,
            Some(|ty| OwnedData::Newtype(Box::new(ty))),
            Some(|ty| OwnedData::Tuple(Box::new([ty]))),
            Some(|ty| OwnedData::Tuple(Box::new([OwnedDataModelType::Unit, ty]))),
            Some(|ty| {
                OwnedData::Struct(Box::new([OwnedNamedField {
                    name: "f".into(),
                    ty,
                }]))
            }),
        ];
        for variant in variants {
            let at = nested(128, variant);
            let over = nested(129, variant);
            let data = [0; 129];

            let value = from_slice_dyn(&at, &data).unwrap();
            let res = from_slice_dyn(&over, &data);
            assert_eq!(res, Err(Error::LimitExceeded));

            let config = DeserializeConfig {
                max_depth: 129,
                ..Default::default()
            };
            let deeper = from_slice_dyn_with_config(&over, &data, &config).unwrap();

            // Encoding has the same limit.
            assert!(to_stdvec_dyn(&at, &value).is_ok());
            let res = to_stdvec_dyn(&over, &deeper);
            assert_eq!(res, Err(crate::ser::Error::LimitExceeded));
            let config = SerializeConfig { max_depth: 129 };
            assert!(to_stdvec_dyn_with_config(&over, &deeper, &config).is_ok());
        }

        // The depth is restored after each element.
        let schema = <Vec<Option<u8>>>::SCHEMA.into();
        let data = [&[200, 1][..], &[1; 400]].concat();
        let value = from_slice_dyn(&schema, &data).unwrap();
        assert_eq!(to_stdvec_dyn(&schema, &value), Ok(data));
    }
}
