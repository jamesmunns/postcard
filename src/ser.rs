use std::num::TryFromIntError;

use postcard::experimental::schema::{OwnedNamedType, OwnedSdmTy, SdmTy, Varint};
use serde_json::Value;
use varint::{varint_max, varint_u128, varint_u16, varint_u32, varint_u64, varint_usize, zig_zag_i128, zig_zag_i16, zig_zag_i32, zig_zag_i64};

#[derive(Debug, PartialEq)]
pub enum Error {
    SchemaMismatch,
    /// Limitations of using serde_json::Value for now
    ShouldSupportButDont,
    Unsupported,
}

pub fn to_stdvec_dyn(
    schema: &OwnedNamedType,
    value: &Value,
) -> Result<Vec<u8>, Error> {
    let mut out = vec![];

    ser_named_type(&schema.ty, value, &mut out)?;

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

fn ser_named_type(
    ty: &OwnedSdmTy,
    value: &Value,
    out: &mut Vec<u8>,
) -> Result<(), Error> {
    match ty {
        OwnedSdmTy::Bool => {
            let val = value.as_bool().right()?;
            out.push(if val { 0x01 } else { 0x00 });
        },
        OwnedSdmTy::I8 => {
            let val = value.as_i64().right()?;
            let val = i8::try_from(val)?;
            out.push(val as u8);
        },
        OwnedSdmTy::U8 => {
            let val = value.as_u64().right()?;
            let val = u8::try_from(val)?;
            out.push(val);
        },
        OwnedSdmTy::Varint(v) => match v {
            Varint::I16 => {
                let val = value.as_i64().right()?;
                let val = i16::try_from(val)?;
                let val = zig_zag_i16(val);
                let mut buf = [0u8; varint_max::<i16>()];
                let used = varint_u16(val, &mut buf);
                out.extend_from_slice(used);
            },
            Varint::I32 => {
                let val = value.as_i64().right()?;
                let val = i32::try_from(val)?;
                let val = zig_zag_i32(val);
                let mut buf = [0u8; varint_max::<i32>()];
                let used = varint_u32(val, &mut buf);
                out.extend_from_slice(used);
            },
            Varint::I64 => {
                let val = value.as_i64().right()?;
                let val = zig_zag_i64(val);
                let mut buf = [0u8; varint_max::<i64>()];
                let used = varint_u64(val, &mut buf);
                out.extend_from_slice(used);
            },
            Varint::I128 => {
                let val = value.as_i64().right()?;
                let val = i128::from(val);
                let val = zig_zag_i128(val);
                let mut buf = [0u8; varint_max::<i128>()];
                let used = varint_u128(val, &mut buf);
                out.extend_from_slice(used);
            },
            Varint::U16 => {
                let val = value.as_u64().right()?;
                let val = u16::try_from(val)?;
                let mut buf = [0u8; varint_max::<u16>()];
                let used = varint_u16(val, &mut buf);
                out.extend_from_slice(used);
            },
            Varint::U32 => {
                let val = value.as_u64().right()?;
                let val = u32::try_from(val)?;
                let mut buf = [0u8; varint_max::<u32>()];
                let used = varint_u32(val, &mut buf);
                out.extend_from_slice(used);
            },
            Varint::U64 => {
                let val = value.as_u64().right()?;
                let mut buf = [0u8; varint_max::<u64>()];
                let used = varint_u64(val, &mut buf);
                out.extend_from_slice(used);
            },
            Varint::U128 => {
                let val = value.as_u64().right()?;
                let val = u128::from(val);
                let mut buf = [0u8; varint_max::<u128>()];
                let used = varint_u128(val, &mut buf);
                out.extend_from_slice(used);
            },
            Varint::Usize => {
                let val = value.as_u64().right()?;
                let val = usize::try_from(val)?;
                let mut buf = [0u8; varint_max::<usize>()];
                let used = varint_usize(val, &mut buf);
                out.extend_from_slice(used);
            },
            Varint::Isize => {
                let val = value.as_i64().right()?;
                let mut buf;
                let used;

                // hax
                #[cfg(target_pointer_width = "16")]
                {
                    let val = i16::try_from(val)?;
                    let val = zig_zag_i16(val);
                    buf = [0u8; varint_max::<i16>()];
                    used = varint_u16(val, &mut buf);
                }
                #[cfg(target_pointer_width = "32")]
                {
                    let val = i32::try_from(val)?;
                    let val = zig_zag_i32(val);
                    buf = [0u8; varint_max::<i32>()];
                    used = varint_u32(val, &mut buf);
                }
                #[cfg(target_pointer_width = "64")]
                {
                    let val = zig_zag_i64(val);
                    buf = [0u8; varint_max::<i64>()];
                    used = varint_u64(val, &mut buf);
                }
                out.extend_from_slice(used);
            },
        },
        OwnedSdmTy::F32 => {
            let val = value.as_f64().right()?;
            let val = val as f32; // todo
            let val = val.to_le_bytes();
            out.extend_from_slice(&val);
        },
        OwnedSdmTy::F64 => {
            let val = value.as_f64().right()?;
            let val = val.to_le_bytes();
            out.extend_from_slice(&val);
        },
        OwnedSdmTy::String | OwnedSdmTy::Char => {
            let val = value.as_str().right()?;

            // First add len
            let len = val.len();
            let mut buf = [0u8; varint_max::<usize>()];
            let used = varint_usize(len, &mut buf);
            out.extend_from_slice(used);

            // Then add payload
            out.extend_from_slice(val.as_bytes());
        },
        OwnedSdmTy::ByteArray => {
            let val = value.as_array().right()?;

            // First add len
            let len = val.len();
            let mut buf = [0u8; varint_max::<usize>()];
            let used = varint_usize(len, &mut buf);
            out.extend_from_slice(used);

            // Then add values
            for b in val {
                let val = b.as_u64().right()?;
                let val = u8::try_from(val)?;
                out.push(val);
            }
        },
        OwnedSdmTy::Option(nt) => {
            if value.is_null() {
                out.push(0x00);
            } else {
                out.push(0x01);
                ser_named_type(&nt.ty, value, out)?;
            }
        },
        OwnedSdmTy::Unit => {},
        OwnedSdmTy::UnitStruct => {},
        OwnedSdmTy::UnitVariant => {},
        OwnedSdmTy::NewtypeStruct(nt) | OwnedSdmTy::NewtypeVariant(nt) => {
            ser_named_type(&nt.ty, value, out)?;
        },
        OwnedSdmTy::Seq(nt) => {
            let val = value.as_array().right()?;

            // First add len
            let len = val.len();
            let mut buf = [0u8; varint_max::<usize>()];
            let used = varint_usize(len, &mut buf);
            out.extend_from_slice(used);

            // Then add values
            for b in val {
                ser_named_type(&nt.ty, b, out)?;
            }
        },
        OwnedSdmTy::Tuple(nts) | OwnedSdmTy::TupleStruct(nts) | OwnedSdmTy::TupleVariant(nts) => {
            // Tuples with arity of 1 are not arrays, but instead just a single object
            if nts.len() == 1 {
                return ser_named_type(&nts[0].ty, value, out);
            }

            let val = value.as_array().right()?;

            if val.len() != nts.len() {
                return Err(Error::SchemaMismatch);
            }

            for (nt, val) in nts.iter().zip(val.iter()) {
                ser_named_type(&nt.ty, val, out)?;
            }
        },
        OwnedSdmTy::Map { key, val } => {
            // TODO: impling blind because we can't test this, oops
            //
            // TODO: There's also a mismatch here because serde_json::Value requires
            // keys to be strings, when postcard doesn't.
            if key.ty != OwnedSdmTy::String {
                return Err(Error::ShouldSupportButDont)
            }

            let obj = value.as_object().right()?;

            // First add len
            let len = obj.len();
            let mut buf = [0u8; varint_max::<usize>()];
            let used = varint_usize(len, &mut buf);
            out.extend_from_slice(used);

            // Then for each pair, serialize key then val
            for (k, v) in obj.iter() {
                // KEY
                //
                // First add len
                let len = k.len();
                let used = varint_usize(len, &mut buf);
                out.extend_from_slice(used);

                // Then add payload
                out.extend_from_slice(k.as_bytes());

                // VALUE
                ser_named_type(&val.ty, v, out)?;
            }
        },
        OwnedSdmTy::Struct(nvs) | OwnedSdmTy::StructVariant(nvs) => {
            let val = value.as_object().right()?;

            if val.len() != nvs.len() {
                return Err(Error::SchemaMismatch);
            }

            for field in nvs.iter() {
                let v = val.get(&field.name).right()?;
                ser_named_type(&field.ty.ty, v, out)?;
            }
        },
        OwnedSdmTy::Enum(nvars) => {
            // This is a bit serde_json::Value specific, if we make our own value
            // type we might be able to handle this "better"

            // Is this a valueless variant?
            if let Some(s) = value.as_str() {
                // Is there a unit variant that matches this name?
                let (idx, evar) = nvars.iter().enumerate().find(|(_i, v)| v.name == s).right()?;
                if evar.ty != OwnedSdmTy::UnitVariant {
                    return Err(Error::SchemaMismatch);
                }

                // cool, we found it, serialize as a varint usize
                let mut buf = [0u8; varint_max::<usize>()];
                let used = varint_usize(idx, &mut buf);
                out.extend_from_slice(used);
            } else if let Some(o) = value.as_object() {
                // This should be an object with exactly one key
                if o.len() != 1 {
                    return Err(Error::SchemaMismatch);
                }
                let (k, v) = o.iter().next().right()?;
                let (idx, evar) = nvars.iter().enumerate().find(|(_i, v)| &v.name == k).right()?;

                // cool, we found it, serialize as a varint usize
                let mut buf = [0u8; varint_max::<usize>()];
                let used = varint_usize(idx, &mut buf);
                out.extend_from_slice(used);

                // then serialize the value
                ser_named_type(&evar.ty, v, out)?;
            } else {
                return Err(Error::SchemaMismatch);
            }
        },
    }
    Ok(())
}

pub(crate) mod varint {
    // copy and paste from postcard

    /// Returns the maximum number of bytes required to encode T.
    pub const fn varint_max<T: Sized>() -> usize {
        const BITS_PER_BYTE: usize = 8;
        const BITS_PER_VARINT_BYTE: usize = 7;

        // How many data bits do we need for this type?
        let bits = core::mem::size_of::<T>() * BITS_PER_BYTE;

        // We add (BITS_PER_VARINT_BYTE - 1), to ensure any integer divisions
        // with a remainder will always add exactly one full byte, but
        // an evenly divided number of bits will be the same
        let roundup_bits = bits + (BITS_PER_VARINT_BYTE - 1);

        // Apply division, using normal "round down" integer division
        roundup_bits / BITS_PER_VARINT_BYTE
    }

    #[inline]
    pub fn varint_usize(n: usize, out: &mut [u8; varint_max::<usize>()]) -> &mut [u8] {
        let mut value = n;
        for i in 0..varint_max::<usize>() {
            out[i] = value.to_le_bytes()[0];
            if value < 128 {
                return &mut out[..=i];
            }

            out[i] |= 0x80;
            value >>= 7;
        }
        debug_assert_eq!(value, 0);
        &mut out[..]
    }

    #[inline]
    pub fn varint_u16(n: u16, out: &mut [u8; varint_max::<u16>()]) -> &mut [u8] {
        let mut value = n;
        for i in 0..varint_max::<u16>() {
            out[i] = value.to_le_bytes()[0];
            if value < 128 {
                return &mut out[..=i];
            }

            out[i] |= 0x80;
            value >>= 7;
        }
        debug_assert_eq!(value, 0);
        &mut out[..]
    }

    #[inline]
    pub fn varint_u32(n: u32, out: &mut [u8; varint_max::<u32>()]) -> &mut [u8] {
        let mut value = n;
        for i in 0..varint_max::<u32>() {
            out[i] = value.to_le_bytes()[0];
            if value < 128 {
                return &mut out[..=i];
            }

            out[i] |= 0x80;
            value >>= 7;
        }
        debug_assert_eq!(value, 0);
        &mut out[..]
    }

    #[inline]
    pub fn varint_u64(n: u64, out: &mut [u8; varint_max::<u64>()]) -> &mut [u8] {
        let mut value = n;
        for i in 0..varint_max::<u64>() {
            out[i] = value.to_le_bytes()[0];
            if value < 128 {
                return &mut out[..=i];
            }

            out[i] |= 0x80;
            value >>= 7;
        }
        debug_assert_eq!(value, 0);
        &mut out[..]
    }

    #[inline]
    pub fn varint_u128(n: u128, out: &mut [u8; varint_max::<u128>()]) -> &mut [u8] {
        let mut value = n;
        for i in 0..varint_max::<u128>() {
            out[i] = value.to_le_bytes()[0];
            if value < 128 {
                return &mut out[..=i];
            }

            out[i] |= 0x80;
            value >>= 7;
        }
        debug_assert_eq!(value, 0);
        &mut out[..]
    }

    pub fn zig_zag_i16(n: i16) -> u16 {
        ((n << 1) ^ (n >> 15)) as u16
    }

    pub fn zig_zag_i32(n: i32) -> u32 {
        ((n << 1) ^ (n >> 31)) as u32
    }

    pub fn zig_zag_i64(n: i64) -> u64 {
        ((n << 1) ^ (n >> 63)) as u64
    }

    pub fn zig_zag_i128(n: i128) -> u128 {
        ((n << 1) ^ (n >> 127)) as u128
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::to_stdvec_dyn;
    use postcard::experimental::schema::Schema;

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
        // TODO: struct variants are broken in the Schema derive in
        // stable postcard, tho it is fixed on the main branch.
        // Zeta { a: u8, b: bool },
    }

    #[test]
    fn ints() {
        let pos = json!(45);

        let t = to_stdvec_dyn(u8::SCHEMA, &pos).unwrap();
        assert_eq!(t, vec![45]);

        let t = to_stdvec_dyn(u16::SCHEMA, &pos).unwrap();
        assert_eq!(t, vec![45]);

        let t = to_stdvec_dyn(u32::SCHEMA, &pos).unwrap();
        assert_eq!(t, vec![45]);

        let t = to_stdvec_dyn(u64::SCHEMA, &pos).unwrap();
        assert_eq!(t, vec![45]);

        let t = to_stdvec_dyn(u128::SCHEMA, &pos).unwrap();
        assert_eq!(t, vec![45]);

        let neg = json!(-45);

        let t = to_stdvec_dyn(i8::SCHEMA, &neg).unwrap();
        // i8s are serialized as-is
        assert_eq!(t, vec![211]);

        // All other types get zig-zag'd first
        let t = to_stdvec_dyn(i16::SCHEMA, &neg).unwrap();
        assert_eq!(t, vec![89]);

        let t = to_stdvec_dyn(i32::SCHEMA, &neg).unwrap();
        assert_eq!(t, vec![89]);

        let t = to_stdvec_dyn(i64::SCHEMA, &neg).unwrap();
        assert_eq!(t, vec![89]);

        let t = to_stdvec_dyn(i128::SCHEMA, &neg).unwrap();
        assert_eq!(t, vec![89]);

        let pos = json!(128);
        let t = to_stdvec_dyn(u16::SCHEMA, &pos).unwrap();
        assert_eq!(t, vec![128, 1]);

        let pos = json!(-65);
        let t = to_stdvec_dyn(i16::SCHEMA, &pos).unwrap();
        assert_eq!(t, vec![129, 1]);
    }

    #[test]
    fn opts() {
        let bys = serde_json::to_value(Some(5i16)).unwrap();
        let t = to_stdvec_dyn(Option::<i16>::SCHEMA, &bys).unwrap();
        assert_eq!(t, [0x01, 0x0A]);
        let byn = serde_json::to_value(Option::<i16>::None).unwrap();
        let t = to_stdvec_dyn(Option::<i16>::SCHEMA, &byn).unwrap();
        assert_eq!(t, [0x00]);
    }

    #[test]
    fn strs() {
        let s = json!("Hello, world!");
        let t = to_stdvec_dyn(String::SCHEMA, &s).unwrap();
        let mut exp = vec![13];
        exp.extend_from_slice(b"Hello, world!");
        assert_eq!(exp, t);
    }

    #[test]
    fn seqs() {
        let s = json!([1, 2, 3, 4]);
        let t = to_stdvec_dyn(Vec::<u16>::SCHEMA, &s).unwrap();
        assert_eq!(vec![4, 1, 2, 3, 4], t);
    }

    #[test]
    fn tups() {
        let byt = serde_json::to_value((true, 5u8, 1.0f32)).unwrap();
        type Tup1 = (bool, u8, f32);
        let t = to_stdvec_dyn(Tup1::SCHEMA, &byt).unwrap();
        assert_eq!(vec![1, 5, 0, 0, 128, 63], t);
    }

    #[test]
    fn structs() {
        let bysct = serde_json::to_value(Struct1 { x: false, y: 1000, z: 4.0 }).unwrap();
        let t = to_stdvec_dyn(Struct1::SCHEMA, &bysct).unwrap();
        assert_eq!(vec![0, 232, 7, 0, 0, 0, 0, 0, 0, 16, 64], t);

        let bysct = serde_json::to_value(TupStruct1(1)).unwrap();
        let t = to_stdvec_dyn(TupStruct1::SCHEMA, &bysct).unwrap();
        assert_eq!(vec![1], t);

        let bysct = serde_json::to_value(TupStruct2(1, 2)).unwrap();
        let t = to_stdvec_dyn(TupStruct2::SCHEMA, &bysct).unwrap();
        assert_eq!(vec![1, 2], t);

        let bysct = serde_json::to_value(UnitStruct).unwrap();
        let t = to_stdvec_dyn(UnitStruct::SCHEMA, &bysct).unwrap();
        assert_eq!(Vec::<u8>::new(), t);
    }

    #[test]
    fn enums() {
        let bye = serde_json::to_value(Enum1::Alpha).unwrap();
        let t = to_stdvec_dyn(Enum1::SCHEMA, &bye).unwrap();
        assert_eq!(vec![0], t);

        let bye = serde_json::to_value(Enum1::Beta(4)).unwrap();
        let t = to_stdvec_dyn(Enum1::SCHEMA, &bye).unwrap();
        assert_eq!(vec![1, 4], t);

        let bye = serde_json::to_value(Enum1::Gamma(vec![1, 2, 3])).unwrap();
        let t = to_stdvec_dyn(Enum1::SCHEMA, &bye).unwrap();
        assert_eq!(vec![2, 3, 1, 2, 3], t);

        let bye = serde_json::to_value(Enum1::Delta(Struct1 { x: false, y: 1000, z: 4.0 })).unwrap();
        let t = to_stdvec_dyn(Enum1::SCHEMA, &bye).unwrap();
        assert_eq!(vec![3, 0, 232, 7, 0, 0, 0, 0, 0, 0, 16, 64], t);

        let bye = serde_json::to_value(Enum1::Epsilon(8, false)).unwrap();
        let t = to_stdvec_dyn(Enum1::SCHEMA, &bye).unwrap();
        assert_eq!(vec![4, 8, 0], t);
    }

    // TODO: we don't implement schema for map types like HashMap, BTreeMap, heapless::IndexMap, or
    // similar types. We should fix that.
    //
    // #[test]
    // fn maps() {
    //     type Map1 = HashMap<String, i64>;
    //     let mut map: Map1 = HashMap::new();
    //     map.insert("bib".to_string(), 10);
    //     map.insert("bim".to_string(), 20);
    //     map.insert("bap".to_string(), 30);
    //     let bym = serde_json::to_value(map).unwrap();
    //     let t = to_stdvec_dyn(Map1::SCHEMA, &bym).unwrap();
    // }

    // Figuring out how serde_json handles various types
    #[test]
    fn serde_j() {
        let bys = serde_json::to_value(Some(5i16)).unwrap();
        assert_eq!(bys.as_i64().unwrap(), 5);

        let byn = serde_json::to_value(Option::<i16>::None).unwrap();
        assert!(byn.is_null());

        let byt = serde_json::to_value((true, 5u8, 1.0f32)).unwrap();
        assert!(byt.is_array());

        let bysct = serde_json::to_value(Struct1 { x: false, y: 1000, z: 4.0 }).unwrap();
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
