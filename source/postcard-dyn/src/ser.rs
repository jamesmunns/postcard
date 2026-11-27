use std::{convert::Infallible, fmt::Display, num::TryFromIntError};

use postcard_core::ser::{
    try_push_bool, try_push_bytes, try_push_char, try_push_discriminant, try_push_f32,
    try_push_f64, try_push_i8, try_push_i16, try_push_i32, try_push_i64, try_push_i128,
    try_push_length, try_push_option_none, try_push_option_some, try_push_str, try_push_u8,
    try_push_u16, try_push_u32, try_push_u64, try_push_u128,
};
use postcard_schema::schema::{OwnedData, OwnedDataModelType};
use serde_json::{Number, Value};

/// The error returned when encoding a message.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    SchemaMismatch,
    /// Limitations of using serde_json::Value for now
    ShouldSupportButDont,
    /// A limit of the [`SerializeConfig`] was exceeded.
    LimitExceeded,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SchemaMismatch => write!(
                f,
                "provided schema did not match the shape of provided data"
            ),
            Error::ShouldSupportButDont => write!(
                f,
                "reached a limitation of using serde_json::Value as the encoding source"
            ),
            Error::LimitExceeded => write!(f, "exceeded a configured limit"),
        }
    }
}

impl std::error::Error for Error {}

/// The limits used by [`to_stdvec_dyn_with_config`].
///
/// To change a limit, start from the default:
///
/// ```rust
/// use postcard_dyn::SerializeConfig;
///
/// let mut config = SerializeConfig::default();
/// config.max_depth = 32;
/// ```
// TODO: Limit the output length? It can't be much larger than `value` is in memory.
// TODO: Limit zero-sized elements? A `Vec<()>` of 129 can be encoded, but the default
// `DeserializeConfig` doesn't decode it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SerializeConfig {
    /// The maximum depth of nested types in the schema. Defaults to 128.
    pub max_depth: usize,
}

impl Default for SerializeConfig {
    fn default() -> Self {
        Self { max_depth: 128 }
    }
}

/// Encode `value` as a message with the given schema, with the default
/// [`SerializeConfig`].
pub fn to_stdvec_dyn(schema: &OwnedDataModelType, value: &Value) -> Result<Vec<u8>, Error> {
    to_stdvec_dyn_with_config(schema, value, &SerializeConfig::default())
}

/// Encode `value` as a message with the given schema.
pub fn to_stdvec_dyn_with_config(
    schema: &OwnedDataModelType,
    value: &Value,
    config: &SerializeConfig,
) -> Result<Vec<u8>, Error> {
    let mut out = vec![];

    ser_named_type(schema, value, &mut out, config.max_depth)?;

    Ok(out)
}

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

impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        Self::SchemaMismatch
    }
}

/// Pushing to a `Vec<u8>` can't fail.
fn infallible(res: Result<(), Infallible>) {
    let Ok(()) = res;
}

fn ser_named_type(
    ty: &OwnedDataModelType,
    value: &Value,
    out: &mut Vec<u8>,
    depth: usize,
) -> Result<(), Error> {
    let depth = depth.checked_sub(1).ok_or(Error::LimitExceeded)?;
    match ty {
        OwnedDataModelType::Bool => {
            let val = value.as_bool().right()?;
            infallible(try_push_bool(out, val));
        }
        OwnedDataModelType::I8 => {
            let val = value.as_i64().right()?;
            let val = i8::try_from(val)?;
            infallible(try_push_i8(out, val));
        }
        OwnedDataModelType::U8 => {
            let val = value.as_u64().right()?;
            let val = u8::try_from(val)?;
            infallible(try_push_u8(out, val));
        }
        OwnedDataModelType::I16 => {
            let val = value.as_i64().right()?;
            let val = i16::try_from(val)?;
            infallible(try_push_i16(out, val));
        }
        OwnedDataModelType::I32 => {
            let val = value.as_i64().right()?;
            let val = i32::try_from(val)?;
            infallible(try_push_i32(out, val));
        }
        OwnedDataModelType::I64 => {
            let val = value.as_i64().right()?;
            infallible(try_push_i64(out, val));
        }
        OwnedDataModelType::I128 => {
            let val = value.as_i64().right()?;
            infallible(try_push_i128(out, i128::from(val)));
        }
        OwnedDataModelType::U16 => {
            let val = value.as_u64().right()?;
            let val = u16::try_from(val)?;
            infallible(try_push_u16(out, val));
        }
        OwnedDataModelType::U32 => {
            let val = value.as_u64().right()?;
            let val = u32::try_from(val)?;
            infallible(try_push_u32(out, val));
        }
        OwnedDataModelType::U64 => {
            let val = value.as_u64().right()?;
            infallible(try_push_u64(out, val));
        }
        OwnedDataModelType::U128 => {
            let val = value.as_u64().right()?;
            infallible(try_push_u128(out, u128::from(val)));
        }
        OwnedDataModelType::F32 => {
            let val = value.as_f64().right()?;
            let val = val as f32; // todo
            infallible(try_push_f32(out, val));
        }
        OwnedDataModelType::F64 => {
            let val = value.as_f64().right()?;
            infallible(try_push_f64(out, val));
        }
        OwnedDataModelType::Char => {
            let val = value.as_str().right()?;
            // A char is a string holding exactly one char.
            let mut chars = val.chars();
            let (Some(c), None) = (chars.next(), chars.next()) else {
                return Err(Error::SchemaMismatch);
            };
            infallible(try_push_char(out, c));
        }
        OwnedDataModelType::String { .. } => {
            let val = value.as_str().right()?;
            infallible(try_push_str(out, val));
        }
        OwnedDataModelType::ByteArray { .. } => {
            let val = value.as_array().right()?;
            let bytes = val
                .iter()
                .map(|b| Ok(u8::try_from(b.as_u64().right()?)?))
                .collect::<Result<Vec<u8>, Error>>()?;
            infallible(try_push_bytes(out, &bytes));
        }
        OwnedDataModelType::Option(ty) => {
            if value.is_null() {
                infallible(try_push_option_none(out));
            } else {
                infallible(try_push_option_some(out));
                ser_named_type(ty, value, out, depth)?;
            }
        }
        OwnedDataModelType::Unit => {}
        OwnedDataModelType::Struct {
            name: _,
            data: OwnedData::Unit,
        } => {}
        OwnedDataModelType::Struct {
            name: _,
            data: OwnedData::Newtype(ty),
        } => {
            ser_named_type(ty, value, out, depth)?;
        }
        OwnedDataModelType::Seq { element: ty, .. } => {
            let val = value.as_array().right()?;
            // First add len
            infallible(try_push_length(out, val.len()));
            // Then add values
            for b in val {
                ser_named_type(ty, b, out, depth)?;
            }
        }
        OwnedDataModelType::Tuple(tys)
        | OwnedDataModelType::Struct {
            name: _,
            data: OwnedData::Tuple(tys),
        } => {
            // Tuples with arity of 1 are not arrays, but instead just a single object
            if tys.len() == 1 {
                return ser_named_type(&tys[0], value, out, depth);
            }
            let val = value.as_array().right()?;
            if val.len() != tys.len() {
                return Err(Error::SchemaMismatch);
            }
            for (ty, val) in tys.iter().zip(val.iter()) {
                ser_named_type(ty, val, out, depth)?;
            }
        }
        OwnedDataModelType::Map { key, val, .. } => {
            let obj = value.as_object().right()?;
            // First add len
            infallible(try_push_length(out, obj.len()));

            // Recover the keys, then put the entries back in key order, as the
            // order `serde_json` keeps them in is not the one postcard uses.
            let mut entries = Vec::with_capacity(obj.len());
            for (k, v) in obj.iter() {
                entries.push((map_key(key, k)?, v));
            }
            entries.sort_by_key(|((order, _), _)| *order);

            // Then for each pair, serialize key then val
            for ((_, k), v) in entries.iter() {
                // KEY
                ser_named_type(key, k, out, depth)?;
                // VALUE
                ser_named_type(val, v, out, depth)?;
            }
        }
        OwnedDataModelType::Struct {
            name: _,
            data: OwnedData::Struct(nvs),
        } => {
            let val = value.as_object().right()?;

            if val.len() != nvs.len() {
                return Err(Error::SchemaMismatch);
            }

            for field in nvs.iter() {
                let v = val.get(field.name.as_ref()).right()?;
                ser_named_type(&field.ty, v, out, depth)?;
            }
        }
        OwnedDataModelType::Enum {
            name: _,
            variants: nvars,
        } => {
            // This is a bit serde_json::Value specific, if we make our own value
            // type we might be able to handle this "better"

            // Is this a valueless variant?
            if let Some(s) = value.as_str() {
                // Is there a unit variant that matches this name?
                let (idx, evar) = nvars
                    .iter()
                    .enumerate()
                    .find(|(_i, v)| *v.name == *s)
                    .right()?;
                if evar.data != OwnedData::Unit {
                    return Err(Error::SchemaMismatch);
                }

                // Found it; push its index as the discriminant.
                let idx = u32::try_from(idx)?;
                infallible(try_push_discriminant(out, idx));
            } else if let Some(o) = value.as_object() {
                // This should be an object with exactly one key
                if o.len() != 1 {
                    return Err(Error::SchemaMismatch);
                }
                let (k, v) = o.iter().next().right()?;
                let (idx, evar) = nvars
                    .iter()
                    .enumerate()
                    .find(|(_i, v)| *v.name == *k)
                    .right()?;

                // Found it; push its index as the discriminant.
                let idx = u32::try_from(idx)?;
                infallible(try_push_discriminant(out, idx));

                // then serialize the value
                match &evar.data {
                    OwnedData::Unit => {
                        // Nothing to do
                    }
                    OwnedData::Newtype(ty) => {
                        ser_named_type(ty, v, out, depth)?;
                    }
                    OwnedData::Tuple(tys) => {
                        // Tuples with arity of 1 are not arrays, but instead just a single object
                        if tys.len() == 1 {
                            return ser_named_type(&tys[0], v, out, depth);
                        }

                        let val = v.as_array().right()?;

                        if val.len() != tys.len() {
                            return Err(Error::SchemaMismatch);
                        }

                        for (ty, val) in tys.iter().zip(val.iter()) {
                            ser_named_type(ty, val, out, depth)?;
                        }
                    }
                    OwnedData::Struct(nvs) => {
                        let val = v.as_object().right()?;

                        if val.len() != nvs.len() {
                            return Err(Error::SchemaMismatch);
                        }

                        for field in nvs.iter() {
                            let v = val.get(field.name.as_ref()).right()?;
                            ser_named_type(&field.ty, v, out, depth)?;
                        }
                    }
                }
            } else {
                return Err(Error::SchemaMismatch);
            }
        }
    }
    Ok(())
}

/// Where a map key sorts, relative to the other keys of the same map.
///
/// `serde_json` keeps map entries in the lexicographic order of the stringified
/// key, which is only the order of the key type itself when the keys really are
/// strings: `"10"` sorts before `"2"`, and a unit variant sorts by name instead
/// of by declaration. Sorting the entries by this before writing them puts them
/// back in the order postcard wrote them in.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MapKeyOrder<'a> {
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    /// Unit variants sort by declaration order, which is the index we write.
    Variant(usize),
    Str(&'a str),
}

/// Recover a single map key, and where it sorts.
///
/// `serde_json::Value` requires map keys to be strings, when postcard doesn't.
/// `serde_json` handles that by encoding primitive keys as their string
/// representation, so we recover the original key by parsing that string back
/// into the type the schema calls for. Key types that `serde_json` also can't
/// represent as a string are still rejected.
fn map_key<'a>(ty: &OwnedDataModelType, key: &'a str) -> Result<(MapKeyOrder<'a>, Value), Error> {
    let out = match ty {
        OwnedDataModelType::String { .. } => {
            (MapKeyOrder::Str(key), Value::String(key.to_string()))
        }
        // Unit variants of an enum are stored as their name, which is exactly
        // what `ser_named_type` expects for an enum.
        OwnedDataModelType::Enum { variants, .. } => {
            let idx = variants.iter().position(|v| *v.name == *key).right()?;
            (MapKeyOrder::Variant(idx), Value::String(key.to_string()))
        }
        OwnedDataModelType::Bool => {
            let val: bool = key.parse().map_err(|_| Error::SchemaMismatch)?;
            (MapKeyOrder::Bool(val), Value::Bool(val))
        }
        OwnedDataModelType::I8
        | OwnedDataModelType::I16
        | OwnedDataModelType::I32
        | OwnedDataModelType::I64
        | OwnedDataModelType::I128 => {
            let val: i64 = key.parse().map_err(|_| Error::SchemaMismatch)?;
            (MapKeyOrder::Signed(val), Value::Number(Number::from(val)))
        }
        OwnedDataModelType::U8
        | OwnedDataModelType::U16
        | OwnedDataModelType::U32
        | OwnedDataModelType::U64
        | OwnedDataModelType::U128 => {
            let val: u64 = key.parse().map_err(|_| Error::SchemaMismatch)?;
            (MapKeyOrder::Unsigned(val), Value::Number(Number::from(val)))
        }
        // TODO: `serde_json` also stores `char` and float keys as strings, but
        // deserialization of those isn't supported as map keys, so accepting
        // them here would give us a serializer the deserializer can't undo.
        _ => return Err(Error::ShouldSupportButDont),
    };

    Ok(out)
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};

    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::to_stdvec_dyn;
    use postcard_schema::Schema;

    #[derive(Serialize, Deserialize, Schema)]
    struct UnitStruct;

    #[derive(Serialize, Deserialize, Schema)]
    struct TupStruct1(u8);

    #[derive(Serialize, Deserialize, Schema)]
    struct TupStruct2(u8, u8);

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
    fn ints() {
        let pos = json!(45);

        let t = to_stdvec_dyn(&u8::SCHEMA.into(), &pos).unwrap();
        assert_eq!(t, vec![45]);

        let t = to_stdvec_dyn(&u16::SCHEMA.into(), &pos).unwrap();
        assert_eq!(t, vec![45]);

        let t = to_stdvec_dyn(&u32::SCHEMA.into(), &pos).unwrap();
        assert_eq!(t, vec![45]);

        let t = to_stdvec_dyn(&u64::SCHEMA.into(), &pos).unwrap();
        assert_eq!(t, vec![45]);

        let t = to_stdvec_dyn(&u128::SCHEMA.into(), &pos).unwrap();
        assert_eq!(t, vec![45]);

        let neg = json!(-45);

        let t = to_stdvec_dyn(&i8::SCHEMA.into(), &neg).unwrap();
        // i8s are serialized as-is
        assert_eq!(t, vec![211]);

        // All other types get zig-zag'd first
        let t = to_stdvec_dyn(&i16::SCHEMA.into(), &neg).unwrap();
        assert_eq!(t, vec![89]);

        let t = to_stdvec_dyn(&i32::SCHEMA.into(), &neg).unwrap();
        assert_eq!(t, vec![89]);

        let t = to_stdvec_dyn(&i64::SCHEMA.into(), &neg).unwrap();
        assert_eq!(t, vec![89]);

        let t = to_stdvec_dyn(&i128::SCHEMA.into(), &neg).unwrap();
        assert_eq!(t, vec![89]);

        let pos = json!(128);
        let t = to_stdvec_dyn(&u16::SCHEMA.into(), &pos).unwrap();
        assert_eq!(t, vec![128, 1]);

        let pos = json!(-65);
        let t = to_stdvec_dyn(&i16::SCHEMA.into(), &pos).unwrap();
        assert_eq!(t, vec![129, 1]);
    }

    #[test]
    fn opts() {
        let bys = serde_json::to_value(Some(5i16)).unwrap();
        let t = to_stdvec_dyn(&Option::<i16>::SCHEMA.into(), &bys).unwrap();
        assert_eq!(t, [0x01, 0x0A]);
        let byn = serde_json::to_value(Option::<i16>::None).unwrap();
        let t = to_stdvec_dyn(&Option::<i16>::SCHEMA.into(), &byn).unwrap();
        assert_eq!(t, [0x00]);
    }

    #[test]
    fn strs() {
        let s = json!("Hello, world!");
        let t = to_stdvec_dyn(&String::SCHEMA.into(), &s).unwrap();
        let mut exp = vec![13];
        exp.extend_from_slice(b"Hello, world!");
        assert_eq!(exp, t);
    }

    #[test]
    fn chars() {
        let t = to_stdvec_dyn(&char::SCHEMA.into(), &json!("é")).unwrap();
        assert_eq!(t, vec![2, 0xC3, 0xA9]);

        for bad in [json!(""), json!("ab")] {
            let res = to_stdvec_dyn(&char::SCHEMA.into(), &bad);
            assert_eq!(res, Err(crate::ser::Error::SchemaMismatch));
        }
    }

    #[test]
    fn seqs() {
        let s = json!([1, 2, 3, 4]);
        let t = to_stdvec_dyn(&Vec::<u16>::SCHEMA.into(), &s).unwrap();
        assert_eq!(vec![4, 1, 2, 3, 4], t);
    }

    #[test]
    fn tups() {
        let byt = serde_json::to_value((true, 5u8, 1.0f32)).unwrap();
        type Tup1 = (bool, u8, f32);
        let t = to_stdvec_dyn(&Tup1::SCHEMA.into(), &byt).unwrap();
        assert_eq!(vec![1, 5, 0, 0, 128, 63], t);
    }

    #[test]
    fn structs() {
        let bysct = serde_json::to_value(Struct1 {
            x: false,
            y: 1000,
            z: 4.0,
        })
        .unwrap();
        let t = to_stdvec_dyn(&Struct1::SCHEMA.into(), &bysct).unwrap();
        assert_eq!(vec![0, 232, 7, 0, 0, 0, 0, 0, 0, 16, 64], t);

        let bysct = serde_json::to_value(TupStruct1(1)).unwrap();
        let t = to_stdvec_dyn(&TupStruct1::SCHEMA.into(), &bysct).unwrap();
        assert_eq!(vec![1], t);

        let bysct = serde_json::to_value(TupStruct2(1, 2)).unwrap();
        let t = to_stdvec_dyn(&TupStruct2::SCHEMA.into(), &bysct).unwrap();
        assert_eq!(vec![1, 2], t);

        let bysct = serde_json::to_value(UnitStruct).unwrap();
        let t = to_stdvec_dyn(&UnitStruct::SCHEMA.into(), &bysct).unwrap();
        assert_eq!(Vec::<u8>::new(), t);
    }

    #[test]
    fn enums() {
        let bye = serde_json::to_value(Enum1::Alpha).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![0], t);

        let bye = serde_json::to_value(Enum1::Beta(4)).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![1, 4], t);

        let bye = serde_json::to_value(Enum1::Gamma(vec![1, 2, 3])).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![2, 3, 1, 2, 3], t);

        let bye = serde_json::to_value(Enum1::Delta(Struct1 {
            x: false,
            y: 1000,
            z: 4.0,
        }))
        .unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![3, 0, 232, 7, 0, 0, 0, 0, 0, 0, 16, 64], t);

        let bye = serde_json::to_value(Enum1::Epsilon(8, false)).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![4, 8, 0], t);

        let bye = serde_json::to_value(Enum1::Zeta { a: 9, b: true }).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![5, 9, 1], t);
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
        assert_eq!(t.len(), 1 + 3 * (4 + 1));
        assert_eq!(t[0], 3);

        // One entry, so the byte order is fixed.
        let bym = json!({"bib": -2});
        let t = to_stdvec_dyn(&Map1::SCHEMA.into(), &bym).unwrap();
        assert_eq!(t, vec![1, 3, b'b', b'i', b'b', 3]);

        // Non-string keys are stored as their string form, which `serde_json`
        // orders lexicographically. The entries are written in key order.
        type Map2 = BTreeMap<u32, u8>;
        let t = to_stdvec_dyn(&Map2::SCHEMA.into(), &json! {{"2": 20, "10": 100}}).unwrap();
        assert_eq!(t, vec![2, 2, 20, 10, 100]);
    }

    // Figuring out how serde_json handles various types
    #[test]
    fn serde_j() {
        let bys = serde_json::to_value(Some(5i16)).unwrap();
        assert_eq!(bys.as_i64().unwrap(), 5);

        let byn = serde_json::to_value(Option::<i16>::None).unwrap();
        assert!(byn.is_null());

        let byt = serde_json::to_value((true, 5u8, 1.0f32)).unwrap();
        assert!(byt.is_array());

        let bysct = serde_json::to_value(Struct1 {
            x: false,
            y: 1000,
            z: 4.0,
        })
        .unwrap();
        assert!(bysct.is_object());

        let bye = serde_json::to_value(Enum1::Alpha).unwrap();
        assert_eq!(bye.as_str().unwrap(), "Alpha");
        let bye = serde_json::to_value(Enum1::Epsilon(1, true)).unwrap();
        assert!(bye.as_object().is_some());

        let bysct = serde_json::to_value(TupStruct1(1)).unwrap();
        assert!(bysct.is_number());

        let bysct = serde_json::to_value(TupStruct2(1, 2)).unwrap();
        assert!(bysct.is_array());

        let bysct = serde_json::to_value(UnitStruct).unwrap();
        assert!(bysct.is_null());

        // uh oh null coalescing
        let byon = serde_json::to_value(Some(())).unwrap();
        assert!(byon.is_null());
    }
}
