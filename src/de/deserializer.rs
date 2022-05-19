use serde::de::{
    self,
    DeserializeSeed,
    IntoDeserializer,
    Visitor,
    // EnumAccess, MapAccess, VariantAccess
};

use crate::error::{Error, Result};
use crate::varint::{varint_max, decoded_len, devarint_u16, devarint_u64, devarint_u32, DitchU16, DitchU32, DitchU64};
use core::marker::PhantomData;

/// A structure for deserializing a postcard message. For now, Deserializer does not
/// implement the same Flavor interface as the serializer does, as messages are typically
/// easier to deserialize in place. This may change in the future for consistency, or
/// to support items that cannot be deserialized in-place, such as compressed message types
pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    cursor: *const u8,
    end: *const u8,
    _pl: PhantomData<&'de [u8]>,
}

impl<'de> Deserializer<'de> {
    /// Obtain a Deserializer from a slice of bytes
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            cursor: input.as_ptr(),
            end: unsafe { input.as_ptr().add(input.len()) },
            _pl: PhantomData,
        }
    }

    pub fn remaining(self) -> &'de [u8] {
        let remain = (self.end as usize) - (self.cursor as usize);
        unsafe {
            core::slice::from_raw_parts(self.cursor, remain)
        }
    }
}

impl<'de> Deserializer<'de> {
    #[inline]
    fn pop(&mut self) -> Result<u8> {
        if self.cursor == self.end {
            Err(Error::DeserializeUnexpectedEnd)
        } else {
            unsafe {
                let res = Ok(*self.cursor);
                self.cursor = self.cursor.add(1);
                res
            }
        }
    }

    #[inline]
    fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8]> {
        let remain = (self.end as usize) - (self.cursor as usize);
        if remain < ct {
            Err(Error::DeserializeUnexpectedEnd)
        } else {
            unsafe {
                let sli = core::slice::from_raw_parts(self.cursor, ct);
                self.cursor = self.cursor.add(ct);
                Ok(sli)
            }
        }
    }

    #[cfg(target_pointer_width = "32")]
    #[inline(always)]
    fn try_take_varint_usize(&mut self) -> Result<usize> {
        self.try_take_varint_u32().map(|u| u as usize)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline(always)]
    fn try_take_varint_usize(&mut self) -> Result<usize> {
        self.try_take_varint_u64().map(|u| u as usize)
    }

    #[inline]
    fn try_take_varint_u16(&mut self) -> Result<u16> {
        const MAX: usize = varint_max::<u16>();
        let mut ditch = DitchU16::default();
        ditch.header = self.pop()?;
        let length = decoded_len(ditch.header);
        if length > MAX {
            return Err(Error::DeserializeBadVarint);
        }
        let lenm1 = length - 1;
        unsafe {
            core::ptr::copy_nonoverlapping(self.try_take_n(lenm1)?.as_ptr(), ditch.body.bytes.as_mut_ptr(), lenm1);
        }
        Ok(devarint_u16(length, ditch))
    }

    #[inline]
    fn try_take_varint_u32(&mut self) -> Result<u32> {
        const MAX: usize = varint_max::<u32>();
        let mut ditch = DitchU32::default();
        ditch.header = self.pop()?;
        let length = decoded_len(ditch.header);
        if length > MAX {
            return Err(Error::DeserializeBadVarint);
        }
        let lenm1 = length - 1;
        unsafe {
            core::ptr::copy_nonoverlapping(self.try_take_n(lenm1)?.as_ptr(), ditch.body.bytes.as_mut_ptr(), lenm1);
        }
        Ok(devarint_u32(length, ditch))
    }

    #[inline]
    fn try_take_varint_u64(&mut self) -> Result<u64> {
        const MAX: usize = varint_max::<u64>();
        let mut ditch = DitchU64::default();
        ditch.header = self.pop()?;
        let length = decoded_len(ditch.header);
        if length > MAX {
            return Err(Error::DeserializeBadVarint);
        }
        let lenm1 = length - 1;
        unsafe {
            core::ptr::copy_nonoverlapping(self.try_take_n(lenm1)?.as_ptr(), ditch.body.bytes.as_mut_ptr(), lenm1);
        }
        Ok(devarint_u64(length, ditch))
    }
}

struct SeqAccess<'a, 'b: 'a> {
    deserializer: &'a mut Deserializer<'b>,
    len: usize,
}

impl<'a, 'b: 'a> serde::de::SeqAccess<'b> for SeqAccess<'a, 'b> {
    type Error = Error;

    #[inline]
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

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct MapAccess<'a, 'b: 'a> {
    deserializer: &'a mut Deserializer<'b>,
    len: usize,
}

impl<'a, 'b: 'a> serde::de::MapAccess<'b> for MapAccess<'a, 'b> {
    type Error = Error;

    #[inline]
    fn next_key_seed<K: DeserializeSeed<'b>>(&mut self, seed: K) -> Result<Option<K::Value>> {
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

    #[inline]
    fn next_value_seed<V: DeserializeSeed<'b>>(&mut self, seed: V) -> Result<V::Value> {
        DeserializeSeed::deserialize(seed, &mut *self.deserializer)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    // Postcard does not support structures not known at compile time
    #[inline]
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // We wont ever support this.
        Err(Error::WontImplement)
    }

    // Take a boolean encoded as a u8
    #[inline]
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

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 1];
        buf[..].copy_from_slice(self.try_take_n(1)?);
        visitor.visit_i8(i8::from_le_bytes(buf))
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.try_take_varint_u16()?;
        visitor.visit_i16(de_zig_zag_i16(v))
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.try_take_varint_u32()?;
        visitor.visit_i32(de_zig_zag_i32(v))
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.try_take_varint_u64()?;
        visitor.visit_i64(de_zig_zag_i64(v))
    }

    #[inline]
    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 16];
        buf[..].copy_from_slice(self.try_take_n(16)?);
        visitor.visit_i128(i128::from_le_bytes(buf))
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.try_take_n(1)?[0])
    }

    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.try_take_varint_u16()?;
        visitor.visit_u16(v)
    }

    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.try_take_varint_u32()?;
        visitor.visit_u32(v)
    }

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.try_take_varint_u64()?;
        visitor.visit_u64(v)
    }

    #[inline]
    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 16];
        buf[..].copy_from_slice(self.try_take_n(16)?);
        visitor.visit_u128(u128::from_le_bytes(buf))
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.try_take_n(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        visitor.visit_f32(f32::from_bits(u32::from_le_bytes(buf)))
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.try_take_n(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        visitor.visit_f64(f64::from_bits(u64::from_le_bytes(buf)))
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let sz = self.try_take_varint_usize()?;
        if sz > 4 {
            return Err(Error::DeserializeBadChar);
        }
        let bytes: &'de [u8] = self.try_take_n(sz)?;
        // we pass the character through string conversion because
        // this handles transforming the array of code units to a 
        // codepoint. we can't use char::from_u32() because it expects
        // an already-processed codepoint.
        let character = core::str::from_utf8(&bytes)
            .map_err(|_| Error::DeserializeBadChar)?
            .chars()
            .next()
            .ok_or(Error::DeserializeBadChar)?;
        visitor.visit_char(character)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let sz = self.try_take_varint_usize()?;
        let bytes: &'de [u8] = self.try_take_n(sz)?;
        let str_sl = core::str::from_utf8(bytes).map_err(|_| Error::DeserializeBadUtf8)?;

        visitor.visit_borrowed_str(str_sl)
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let sz = self.try_take_varint_usize()?;
        let bytes: &'de [u8] = self.try_take_n(sz)?;
        visitor.visit_borrowed_bytes(bytes)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.try_take_n(1)?[0] {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(Error::DeserializeBadOption),
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    // Unit is not actually encoded in Postcard.
    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    // Unit struct means a named value containing no data.
    // Unit structs are not actually encoded in Postcard.
    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.try_take_varint_usize()?;

        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len,
        })
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len,
        })
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.try_take_varint_usize()?;

        visitor.visit_map(MapAccess {
            deserializer: self,
            len,
        })
    }

    #[inline]
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

    #[inline]
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
    #[inline]
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Will not support
        Err(Error::WontImplement)
    }

    #[inline]
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

    #[inline]
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn newtype_variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<V::Value> {
        DeserializeSeed::deserialize(seed, self)
    }

    #[inline]
    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        serde::de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    #[inline]
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

    #[inline]
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let varint = self.try_take_varint_u32()?;
        let v = DeserializeSeed::deserialize(seed, varint.into_deserializer())?;
        Ok((v, self))
    }
}

fn de_zig_zag_i16(n: u16) -> i16 {
    ((n >> 1) as i16) ^ (-((n & 0b1) as i16))
}

fn de_zig_zag_i32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 0b1) as i32))
}

fn de_zig_zag_i64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 0b1) as i64))
}
