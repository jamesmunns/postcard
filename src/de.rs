use crate::error::{Error, Result};
use crate::varint::VarintUsize;
use byteorder::{ByteOrder, LittleEndian};
use cobs::decode_in_place;

use serde::de::{
    self,
    DeserializeSeed,
    IntoDeserializer,
    Visitor,
    // EnumAccess, MapAccess, VariantAccess
};
use serde::Deserialize;

pub fn from_bytes_cobs<'a, T>(s: &'a mut [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let sz = decode_in_place(s).map_err(|_| Error::DeserializeBadEncoding)?;
    from_bytes::<T>(&s[..sz])
}

pub fn take_from_bytes_cobs<'a, T>(s: &'a mut [u8]) -> Result<(T, &'a mut [u8])>
where
    T: Deserialize<'a>,
{
    let sz = decode_in_place(s).map_err(|_| Error::DeserializeBadEncoding)?;
    let (used, unused) = s.split_at_mut(sz);
    Ok((from_bytes::<T>(used)?, unused))
}

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input }
    }
}

pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

pub fn take_from_bytes<'a, T>(s: &'a [u8]) -> Result<(T, &'a [u8])>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok((t, deserializer.input))
}

impl<'de> Deserializer<'de> {
    fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8]> {
        if self.input.len() >= ct {
            let (a, b) = self.input.split_at(ct);
            self.input = b;
            Ok(a)
        } else {
            Err(Error::DeserializeUnexpectedEnd)
        }
    }

    fn try_take_varint(&mut self) -> Result<usize> {
        for i in 0..VarintUsize::varint_usize_max() {
            let val = self.input.get(i).ok_or(Error::DeserializeUnexpectedEnd)?;
            if (val & 0x80) == 0 {
                let (a, b) = self.input.split_at(i + 1);
                self.input = b;
                let mut out = 0usize;
                for byte in a.iter().rev() {
                    out <<= 7;
                    out |= (byte & 0x7F) as usize;
                }
                return Ok(out);
            }
        }

        Err(Error::DeserializeBadVarint)
    }
}

struct SeqAccess<'a, 'b: 'a> {
    deserializer: &'a mut Deserializer<'b>,
    len: usize,
}

impl<'a, 'b: 'a> serde::de::SeqAccess<'b> for SeqAccess<'a, 'b> {
    type Error = Error;

    fn next_element_seed<V: DeserializeSeed<'b>>(&mut self, seed: V) -> Result<Option<V::Value>> {
        if self.len > 0 {
            self.len -= 1;
            Ok(Some(DeserializeSeed::deserialize(
                seed,
                &mut *self.deserializer,
            )?))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Postcard does not support structures not known at compile time
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // We wont ever support this.
        Err(Error::WontImplement)
    }

    // Take a boolean encoded as a u8
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let val = match self.try_take_n(1)?[0] {
            0 => false,
            1 => true,
            _ => return Err(Error::DeserializeBadBool),
        };
        visitor.visit_bool(val)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 1];
        buf[..].copy_from_slice(self.try_take_n(1)?);
        visitor.visit_i8(i8::from_le_bytes(buf))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 2];
        buf[..].copy_from_slice(self.try_take_n(2)?);
        visitor.visit_i16(i16::from_le_bytes(buf))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 4];
        buf[..].copy_from_slice(self.try_take_n(4)?);
        visitor.visit_i32(i32::from_le_bytes(buf))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 8];
        buf[..].copy_from_slice(self.try_take_n(8)?);
        visitor.visit_i64(i64::from_le_bytes(buf))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.try_take_n(1)?[0])
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 2];
        buf[..].copy_from_slice(self.try_take_n(2)?);
        visitor.visit_u16(u16::from_le_bytes(buf))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 4];
        buf[..].copy_from_slice(self.try_take_n(4)?);
        visitor.visit_u32(u32::from_le_bytes(buf))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 8];
        buf[..].copy_from_slice(self.try_take_n(8)?);
        visitor.visit_u64(u64::from_le_bytes(buf))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.try_take_n(4)?;
        visitor.visit_f32(LittleEndian::read_f32(bytes))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.try_take_n(8)?;
        visitor.visit_f64(LittleEndian::read_f64(bytes))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 4];
        let bytes = self.try_take_n(4)?;
        buf.copy_from_slice(bytes);
        let integer = u32::from_le_bytes(buf);
        visitor.visit_char(core::char::from_u32(integer).ok_or(Error::DeserializeBadChar)?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let sz = self.try_take_varint()?;
        let bytes: &'de [u8] = self.try_take_n(sz)?;
        let str_sl = core::str::from_utf8(bytes).map_err(|_| Error::DeserializeBadUtf8)?;

        visitor.visit_borrowed_str(str_sl)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // AJM - in serialize_bytes, we don't write the length first
        // is this asymmetry intended?
        let sz = self.try_take_varint()?;
        let bytes: &'de [u8] = self.try_take_n(sz)?;
        visitor.visit_borrowed_bytes(bytes)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.try_take_n(1)?[0] {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => return Err(Error::DeserializeBadOption),
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    // Unit is not actually encoded in Postcard.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    // Unit struct means a named value containing no data.
    // Unit structs are not actually encoded in Postcard.
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.try_take_varint()?;

        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len: len,
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len: len,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // I plan to implement this, but haven't yet. Open an issue if you'd
        // like it done sooner :)
        Err(Error::NotYetImplemented)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    // As a binary format, Postcard does not encode identifiers
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Will not support
        Err(Error::WontImplement)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Will not support
        Err(Error::WontImplement)
    }
}

impl<'de, 'a> serde::de::VariantAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<V::Value> {
        DeserializeSeed::deserialize(seed, self)
    }

    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        serde::de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        serde::de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

impl<'de, 'a> serde::de::EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let varint = self.try_take_varint()?;
        if varint > 0xFFFF_FFFF {
            return Err(Error::DeserializeBadEnum);
        }
        let v = DeserializeSeed::deserialize(seed, (varint as u32).into_deserializer())?;
        Ok((v, self))
    }
}

// // `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// // through entries of the map.
// impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
//     type Error = Error;

//     fn next_key_seed<K>(&mut self, _seed: K) -> Result<Option<K::Value>>
//     where
//         K: DeserializeSeed<'de>,
//     {
//         // // Check if there are no more entries.
//         // if self.de.peek_char()? == '}' {
//         //     return Ok(None);
//         // }
//         // // Comma is required before every entry except the first.
//         // if !self.first && self.de.next_char()? != ',' {
//         //     return Err(Error::ExpectedMapComma);
//         // }
//         // self.first = false;
//         // // Deserialize a map key.
//         // seed.deserialize(&mut *self.de).map(Some)
//         unimplemented!()
//     }

//     fn next_value_seed<V>(&mut self, _seed: V) -> Result<V::Value>
//     where
//         V: DeserializeSeed<'de>,
//     {
//         // // It doesn't make a difference whether the colon is parsed at the end
//         // // of `next_key_seed` or at the beginning of `next_value_seed`. In this
//         // // case the code is a bit simpler having it here.
//         // if self.de.next_char()? != ':' {
//         //     return Err(Error::ExpectedMapColon);
//         // }
//         // // Deserialize a map value.
//         // seed.deserialize(&mut *self.de)
//         unimplemented!()
//     }
// }

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
    use crate::ser::to_vec;
    use core::fmt::Write;
    use core::ops::Deref;
    use heapless::{consts::*, String, Vec};
    use serde::{Deserialize, Serialize};

    #[test]
    fn de_u8() {
        let output: Vec<u8, U1> = to_vec(&0x05u8).unwrap();
        assert!(&[5] == output.deref());

        let out: u8 = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 0x05);
    }

    #[test]
    fn de_u16() {
        let output: Vec<u8, U2> = to_vec(&0xA5C7u16).unwrap();
        assert!(&[0xC7, 0xA5] == output.deref());

        let out: u16 = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 0xA5C7);
    }

    #[test]
    fn de_u32() {
        let output: Vec<u8, U4> = to_vec(&0xCDAB3412u32).unwrap();
        assert!(&[0x12, 0x34, 0xAB, 0xCD] == output.deref());

        let out: u32 = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 0xCDAB3412u32);
    }

    #[test]
    fn de_u64() {
        let output: Vec<u8, U8> = to_vec(&0x1234_5678_90AB_CDEFu64).unwrap();
        assert!(&[0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12] == output.deref());

        let out: u64 = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 0x1234_5678_90AB_CDEFu64);
    }

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct BasicU8S {
        st: u16,
        ei: u8,
        sf: u64,
        tt: u32,
    }

    #[test]
    fn de_struct_unsigned() {
        let data = BasicU8S {
            st: 0xABCD,
            ei: 0xFE,
            sf: 0x1234_4321_ABCD_DCBA,
            tt: 0xACAC_ACAC,
        };

        let output: Vec<u8, U15> = to_vec(&data).unwrap();

        assert!(
            &[
                0xCD, 0xAB, 0xFE, 0xBA, 0xDC, 0xCD, 0xAB, 0x21, 0x43, 0x34, 0x12, 0xAC, 0xAC, 0xAC,
                0xAC
            ] == output.deref()
        );

        let out: BasicU8S = from_bytes(output.deref()).unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn de_byte_slice() {
        let input: &[u8] = &[1u8, 2, 3, 4, 5, 6, 7, 8];
        let output: Vec<u8, U9> = to_vec(input).unwrap();
        assert_eq!(
            &[0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            output.deref()
        );

        let out: Vec<u8, U128> = from_bytes(output.deref()).unwrap();
        assert_eq!(input, out.deref());

        let mut input: Vec<u8, U1024> = Vec::new();
        for i in 0..1024 {
            input.push((i & 0xFF) as u8).unwrap();
        }
        let output: Vec<u8, U2048> = to_vec(input.deref()).unwrap();
        assert_eq!(&[0x80, 0x08], &output.deref()[..2]);

        assert_eq!(output.len(), 1026);
        for (i, val) in output.deref()[2..].iter().enumerate() {
            assert_eq!((i & 0xFF) as u8, *val);
        }

        let de: Vec<u8, U1024> = from_bytes(output.deref()).unwrap();
        assert_eq!(input.deref(), de.deref());
    }

    #[test]
    fn de_str() {
        let input: &str = "hello, postcard!";
        let output: Vec<u8, U17> = to_vec(input).unwrap();
        assert_eq!(0x10, output.deref()[0]);
        assert_eq!(input.as_bytes(), &output.deref()[1..]);

        let mut input: String<U1024> = String::new();
        for _ in 0..256 {
            write!(&mut input, "abcd").unwrap();
        }
        let output: Vec<u8, U2048> = to_vec(input.deref()).unwrap();
        assert_eq!(&[0x80, 0x08], &output.deref()[..2]);

        assert_eq!(output.len(), 1026);
        for ch in output.deref()[2..].chunks(4) {
            assert_eq!("abcd", core::str::from_utf8(ch).unwrap());
        }

        let de: String<U1024> = from_bytes(output.deref()).unwrap();
        assert_eq!(input.deref(), de.deref());
    }

    #[allow(dead_code)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum BasicEnum {
        Bib,
        Bim,
        Bap,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct EnumStruct {
        eight: u8,
        sixt: u16,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum DataEnum {
        Bib(u16),
        Bim(u64),
        Bap(u8),
        Kim(EnumStruct),
        Chi { a: u8, b: u32 },
        Sho(u16, u8),
    }

    #[test]
    fn enums() {
        let output: Vec<u8, U1> = to_vec(&BasicEnum::Bim).unwrap();
        assert_eq!(&[0x01], output.deref());
        let out: BasicEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, BasicEnum::Bim);

        let output: Vec<u8, U9> = to_vec(&DataEnum::Bim(u64::max_value())).unwrap();
        assert_eq!(
            &[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            output.deref()
        );

        let output: Vec<u8, U3> = to_vec(&DataEnum::Bib(u16::max_value())).unwrap();
        assert_eq!(&[0x00, 0xFF, 0xFF], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Bib(u16::max_value()));

        let output: Vec<u8, U2> = to_vec(&DataEnum::Bap(u8::max_value())).unwrap();
        assert_eq!(&[0x02, 0xFF], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Bap(u8::max_value()));

        let output: Vec<u8, U8> = to_vec(&DataEnum::Kim(EnumStruct {
            eight: 0xF0,
            sixt: 0xACAC,
        }))
        .unwrap();
        assert_eq!(&[0x03, 0xF0, 0xAC, 0xAC,], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(
            out,
            DataEnum::Kim(EnumStruct {
                eight: 0xF0,
                sixt: 0xACAC
            })
        );

        let output: Vec<u8, U8> = to_vec(&DataEnum::Chi {
            a: 0x0F,
            b: 0xC7C7C7C7,
        })
        .unwrap();
        assert_eq!(&[0x04, 0x0F, 0xC7, 0xC7, 0xC7, 0xC7], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(
            out,
            DataEnum::Chi {
                a: 0x0F,
                b: 0xC7C7C7C7
            }
        );

        let output: Vec<u8, U8> = to_vec(&DataEnum::Sho(0x6969, 0x07)).unwrap();
        assert_eq!(&[0x05, 0x69, 0x69, 0x07], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Sho(0x6969, 0x07));
    }

    #[test]
    fn tuples() {
        let output: Vec<u8, U128> = to_vec(&(1u8, 10u32, "Hello!")).unwrap();
        assert_eq!(
            &[1u8, 0x0A, 0x00, 0x00, 0x00, 0x06, b'H', b'e', b'l', b'l', b'o', b'!'],
            output.deref()
        );
        let out: (u8, u32, &str) = from_bytes(output.deref()).unwrap();
        assert_eq!(out, (1u8, 10u32, "Hello!"));
    }

    #[test]
    fn bytes() {
        let x: &[u8; 32] = &[0u8; 32];
        let output: Vec<u8, U128> = to_vec(x).unwrap();
        assert_eq!(output.len(), 32);
        let out: [u8; 32] = from_bytes(output.deref()).unwrap();
        assert_eq!(out, [0u8; 32]);
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct NewTypeStruct(u32);

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct TupleStruct((u8, u16));

    #[test]
    fn structs() {
        let output: Vec<u8, U4> = to_vec(&NewTypeStruct(5)).unwrap();
        assert_eq!(&[0x05, 0x00, 0x00, 0x00], output.deref());
        let out: NewTypeStruct = from_bytes(output.deref()).unwrap();
        assert_eq!(out, NewTypeStruct(5));

        let output: Vec<u8, U3> = to_vec(&TupleStruct((0xA0, 0x1234))).unwrap();
        assert_eq!(&[0xA0, 0x34, 0x12], output.deref());
        let out: TupleStruct = from_bytes(output.deref()).unwrap();
        assert_eq!(out, TupleStruct((0xA0, 0x1234)));
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct RefStruct<'a> {
        bytes: &'a [u8],
        str_s: &'a str,
    }

    #[test]
    fn ref_struct() {
        let message = "hElLo";
        let bytes = [0x01, 0x10, 0x02, 0x20];
        let output: Vec<u8, U11> = to_vec(&RefStruct {
            bytes: &bytes,
            str_s: message,
        })
        .unwrap();

        assert_eq!(
            &[0x04, 0x01, 0x10, 0x02, 0x20, 0x05, b'h', b'E', b'l', b'L', b'o',],
            output.deref()
        );

        let out: RefStruct = from_bytes(output.deref()).unwrap();
        assert_eq!(
            out,
            RefStruct {
                bytes: &bytes,
                str_s: message,
            }
        );
    }

    #[test]
    fn unit() {
        let output: Vec<u8, U1> = to_vec(&()).unwrap();
        assert_eq!(output.len(), 0);
        let out: () = from_bytes(output.deref()).unwrap();
        assert_eq!(out, ());
    }

    #[test]
    fn heapless_data() {
        let mut input: Vec<u8, U4> = Vec::new();
        input.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let output: Vec<u8, U5> = to_vec(&input).unwrap();
        assert_eq!(&[0x04, 0x01, 0x02, 0x03, 0x04], output.deref());
        let out: Vec<u8, U4> = from_bytes(output.deref()).unwrap();
        assert_eq!(out, input);

        let mut input: String<U8> = String::new();
        write!(&mut input, "helLO!").unwrap();
        let output: Vec<u8, U7> = to_vec(&input).unwrap();
        assert_eq!(&[0x06, b'h', b'e', b'l', b'L', b'O', b'!'], output.deref());
        let out: String<U8> = from_bytes(output.deref()).unwrap();
        assert_eq!(input, out);
    }

    #[test]
    fn cobs_test() {
        let message = "hElLo";
        let bytes = [0x01, 0x00, 0x02, 0x20];
        let input = RefStruct {
            bytes: &bytes,
            str_s: message,
        };

        let output: Vec<u8, U11> = to_vec(&input).unwrap();

        let mut encode_buf = [0u8; 32];
        let sz = cobs::encode(output.deref(), &mut encode_buf);
        let out = from_bytes_cobs::<RefStruct>(&mut encode_buf[..sz]).unwrap();

        assert_eq!(input, out);
    }
}
