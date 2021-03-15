use serde::de::{
    self,
    DeserializeSeed,
    IntoDeserializer,
    Visitor,
    // EnumAccess, MapAccess, VariantAccess
};

use crate::error::{Error, Result};
use crate::varint::VarintUsize;

/// A structure for deserializing a postcard message. For now, Deserializer does not
/// implement the same Flavor interface as the serializer does, as messages are typically
/// easier to deserialize in place. This may change in the future for consistency, or
/// to support items that cannot be deserialized in-place, such as compressed message types
pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    pub(crate) input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    /// Obtain a Deserializer from a slice of bytes
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input }
    }
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

struct MapAccess<'a, 'b: 'a> {
    deserializer: &'a mut Deserializer<'b>,
    len: usize,
}

impl<'a, 'b: 'a> serde::de::MapAccess<'b> for MapAccess<'a, 'b> {
    type Error = Error;

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

    fn next_value_seed<V: DeserializeSeed<'b>>(&mut self, seed: V) -> Result<V::Value> {
        DeserializeSeed::deserialize(seed, &mut *self.deserializer)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        false
    }

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

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 16];
        buf[..].copy_from_slice(self.try_take_n(16)?);
        visitor.visit_i128(i128::from_le_bytes(buf))
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

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 16];
        buf[..].copy_from_slice(self.try_take_n(16)?);
        visitor.visit_u128(u128::from_le_bytes(buf))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.try_take_n(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        visitor.visit_f32(f32::from_bits(u32::from_le_bytes(buf)))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.try_take_n(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        visitor.visit_f64(f64::from_bits(u64::from_le_bytes(buf)))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let sz = self.try_take_varint()?;
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
            _ => Err(Error::DeserializeBadOption),
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
            len,
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len,
        })
    }

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

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.try_take_varint()?;

        visitor.visit_map(MapAccess {
            deserializer: self,
            len,
        })
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
