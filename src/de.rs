#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut
)]

use crate::varint::VarintUsize;
use crate::error::{Error, Result};

use serde::Deserialize;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess,
    VariantAccess, Visitor,
};

// use serde::error::{Error, Result};

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input }
    }
}

// By convention, the public API of a Serde deserializer is one or more
// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// depending on what Rust types the deserializer is able to consume as input.
//
// This basic deserializer supports only `from_str`.
pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    // if deserializer.input.is_empty() {
        Ok(t)
    // } else {
    //     Err(Error::TrailingCharacters)
    // }
}

// SERDE IS NOT A PARSING LIBRARY. This impl block defines a few basic parsing
// functions from scratch. More complicated formats may wish to use a dedicated
// parsing library to help implement their Serde deserializer.
impl<'de> Deserializer<'de> {
    fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8]> {
        if self.input.len() >= ct {
            let (a, b) = self.input.split_at(ct);
            self.input = b;
            Ok(a)
        } else {
            Err(Error::ToDo)
        }
    }

    // AJM - this is a hack, I will probably need to figure
    // out how to impl Deserialize for a varint.
    fn try_take_varint(&mut self) -> Result<usize> {
        // println!("{:?}", self.input);
        for i in 0..VarintUsize::varint_usize_max() {
            let val = self.input.get(i).ok_or(Error::ToDo)?;
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

        Err(Error::ToDo)
    }
    // // Look at the first character in the input without consuming it.
    // fn peek_char(&mut self) -> Result<char> {
    //     self.input.chars().next().ok_or(Error::Eof)
    // }

    // // Consume the first character in the input.
    // fn next_char(&mut self) -> Result<char> {
    //     let ch = self.peek_char()?;
    //     self.input = &self.input[ch.len_utf8()..];
    //     Ok(ch)
    // }

    // // Parse the JSON identifier `true` or `false`.
    // fn parse_bool(&mut self) -> Result<bool> {
    //     if self.input.starts_with("true") {
    //         self.input = &self.input["true".len()..];
    //         Ok(true)
    //     } else if self.input.starts_with("false") {
    //         self.input = &self.input["false".len()..];
    //         Ok(false)
    //     } else {
    //         Err(Error::ExpectedBoolean)
    //     }
    // }

    // // Parse a group of decimal digits as an unsigned integer of type T.
    // //
    // // This implementation is a bit too lenient, for example `001` is not
    // // allowed in JSON. Also the various arithmetic operations can overflow and
    // // panic or return bogus data. But it is good enough for example code!
    // fn parse_unsigned<T>(&mut self) -> Result<T>
    // where
    //     T: AddAssign<T> + MulAssign<T> + From<u8>,
    // {
    //     let mut int = match self.next_char()? {
    //         ch @ '0'...'9' => T::from(ch as u8 - b'0'),
    //         _ => {
    //             return Err(Error::ExpectedInteger);
    //         }
    //     };
    //     loop {
    //         match self.input.chars().next() {
    //             Some(ch @ '0'...'9') => {
    //                 self.input = &self.input[1..];
    //                 int *= T::from(10);
    //                 int += T::from(ch as u8 - b'0');
    //             }
    //             _ => {
    //                 return Ok(int);
    //             }
    //         }
    //     }
    // }

    // // Parse a possible minus sign followed by a group of decimal digits as a
    // // signed integer of type T.
    // fn parse_signed<T>(&mut self) -> Result<T>
    // where
    //     T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
    // {
    //     // Optional minus sign, delegate to `parse_unsigned`, negate if negative.
    //     unimplemented!()
    // }

    // // Parse a string until the next '"' character.
    // //
    // // Makes no attempt to handle escape sequences. What did you expect? This is
    // // example code!
    // fn parse_string(&mut self) -> Result<&'de str> {
    //     if self.next_char()? != '"' {
    //         return Err(Error::ExpectedString);
    //     }
    //     match self.input.find('"') {
    //         Some(len) => {
    //             let s = &self.input[..len];
    //             self.input = &self.input[len + 1..];
    //             Ok(s)
    //         }
    //         None => Err(Error::Eof),
    //     }
    // }
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
            Ok(Some(DeserializeSeed::deserialize(seed, &mut *self.deserializer)?))
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

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // match self.peek_char()? {
        //     'n' => self.deserialize_unit(visitor),
        //     't' | 'f' => self.deserialize_bool(visitor),
        //     '"' => self.deserialize_str(visitor),
        //     '0'...'9' => self.deserialize_u64(visitor),
        //     '-' => self.deserialize_i64(visitor),
        //     '[' => self.deserialize_seq(visitor),
        //     '{' => self.deserialize_map(visitor),
        //     _ => Err(Error::Syntax),
        // }
        unimplemented!()
    }

    // Uses the `parse_bool` parsing function defined above to read the JSON
    // identifier `true` or `false` from the input.
    //
    // Parsing refers to looking at the input and deciding that it contains the
    // JSON value `true` or `false`.
    //
    // Deserialization refers to mapping that JSON value into Serde's data
    // model by invoking one of the `Visitor` methods. In the case of JSON and
    // bool that mapping is straightforward so the distinction may seem silly,
    // but in other cases Deserializers sometimes perform non-obvious mappings.
    // For example the TOML format has a Datetime type and Serde's data model
    // does not. In the `toml` crate, a Datetime in the input is deserialized by
    // mapping it to a Serde data model "struct" type with a special name and a
    // single field containing the Datetime represented as a string.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // visitor.visit_bool(self.parse_bool()?)
        unimplemented!()
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
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

    // Float parsing is stupidly hard.
    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Float parsing is stupidly hard.
    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse a string, check that it is one character, call `visit_char`.
        unimplemented!()
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let sz = self.try_take_varint()?;
        let bytes: &'de [u8] = self.try_take_n(sz)?;
        let str_sl = core::str::from_utf8(bytes)
            .map_err(|_| Error::ToDo)?;

        visitor.visit_borrowed_str(str_sl)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // self.deserialize_str(visitor)
        unimplemented!()
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
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

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // if self.input.starts_with("null") {
        //     self.input = &self.input["null".len()..];
        //     visitor.visit_none()
        // } else {
        //     visitor.visit_some(self)
        // }
        unimplemented!()
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // self.deserialize_unit(visitor)
        unimplemented!()
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.try_take_varint()?;

        // println!("{:?}", len);
        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len: len,
        })
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // println!("{}", ::core::mem::size_of::<V>());
        visitor.visit_seq(SeqAccess { deserializer: self, len: len })
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // self.deserialize_seq(visitor)
        unimplemented!()
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // // Parse the opening brace of the map.
        // if self.next_char()? == '{' {
        //     // Give the visitor access to each entry of the map.
        //     let value = visitor.visit_map(CommaSeparated::new(&mut self))?;
        //     // Parse the closing brace of the map.
        //     if self.next_char()? == '}' {
        //         Ok(value)
        //     } else {
        //         Err(Error::ExpectedMapEnd)
        //     }
        // } else {
        //     Err(Error::ExpectedMap)
        // }
        unimplemented!()
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // println!("{}, {:?}", name, fields);
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

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // self.deserialize_str(visitor)
        unimplemented!()
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // self.deserialize_any(visitor)
        unimplemented!()
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

     fn struct_variant<V: Visitor<'de>>(self,
                       fields: &'static [&'static str],
                       visitor: V)
                       -> Result<V::Value> {
        serde::de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

impl<'de, 'a> serde::de::EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let varint = self.try_take_varint()?;
        if varint > 0xFFFF_FFFF {
            return Err(Error::ToDo);
        }
        let v = DeserializeSeed::deserialize(seed, (varint as u32).into_deserializer())?;
        Ok((v, self))
    }
}


// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        // CommaSeparated {
        //     de,
        //     first: true,
        // }
        unimplemented!()
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // // Check if there are no more entries.
        // if self.de.peek_char()? == '}' {
        //     return Ok(None);
        // }
        // // Comma is required before every entry except the first.
        // if !self.first && self.de.next_char()? != ',' {
        //     return Err(Error::ExpectedMapComma);
        // }
        // self.first = false;
        // // Deserialize a map key.
        // seed.deserialize(&mut *self.de).map(Some)
        unimplemented!()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // // It doesn't make a difference whether the colon is parsed at the end
        // // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // // case the code is a bit simpler having it here.
        // if self.de.next_char()? != ':' {
        //     return Err(Error::ExpectedMapColon);
        // }
        // // Deserialize a map value.
        // seed.deserialize(&mut *self.de)
        unimplemented!()
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // // The `deserialize_enum` method parsed a `{` character so we are
        // // currently inside of a map. The seed will be deserializing itself from
        // // the key of the map.
        // let val = seed.deserialize(&mut *self.de)?;
        // // Parse the colon separating map key from value.
        // if self.de.next_char()? == ':' {
        //     Ok((val, self))
        // } else {
        //     Err(Error::ExpectedMapColon)
        // }
        unimplemented!()
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        // Err(Error::ExpectedString)
        unimplemented!()
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        // seed.deserialize(self.de)
        unimplemented!()
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // de::Deserializer::deserialize_map(self.de, visitor)
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use crate::ser::to_vec;
    use super::*;
    use heapless::{
        consts::*,
        String,
        Vec,
    };
    use core::ops::Deref;
    use core::fmt::Write;
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
            tt: 0xACAC_ACAC
        };

        let output: Vec<u8, U15> = to_vec(
            &data).unwrap();

        assert!(&[
            0xCD, 0xAB,
            0xFE,
            0xBA, 0xDC, 0xCD, 0xAB, 0x21, 0x43, 0x34, 0x12,
            0xAC, 0xAC, 0xAC, 0xAC
        ] == output.deref());

        let out: BasicU8S = from_bytes(output.deref()).unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn de_byte_slice() {
        let input: &[u8] = &[1u8, 2, 3, 4, 5, 6, 7, 8];
        let output: Vec<u8, U9> = to_vec(input).unwrap();
        assert_eq!(&[
            0x08,
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08
        ], output.deref());

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

    ////////////
    // AJM - I don't think you can Deserialize a varint :(
    ////////////

    // #[test]
    // fn usize_varint_encode() {
    //     let mut buf = VarintUsize::new_buf();
    //     let res = VarintUsize(1).to_buf(
    //         &mut buf,
    //     );

    //     assert!(&[1] == res);

    //     let res = VarintUsize(usize::max_value()).to_buf(
    //         &mut buf
    //     );

    //     // AJM TODO
    //     if VarintUsize::varint_usize_max() == 5 {
    //         assert_eq!(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F], res);
    //     } else {
    //         assert_eq!(&[0xFF, 0xFF, 0xFF, 0xFF,
    //                      0xFF, 0xFF, 0xFF, 0xFF,
    //                      0xFF, 0x01], res);
    //     }
    // }

    #[allow(dead_code)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum BasicEnum {
        Bib,
        Bim,
        Bap
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
        Chi{ a: u8, b: u32 },
        Sho(u16, u8),
    }

    #[test]
    fn enums() {
        let output: Vec<u8, U1> = to_vec(&BasicEnum::Bim).unwrap();
        assert_eq!(&[0x01], output.deref());
        let out: BasicEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, BasicEnum::Bim);

        let output: Vec<u8, U9> = to_vec(&DataEnum::Bim(u64::max_value())).unwrap();
        assert_eq!(&[
            0x01,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
        ], output.deref());

        let output: Vec<u8, U3> = to_vec(&DataEnum::Bib(u16::max_value())).unwrap();
        assert_eq!(&[
            0x00,
            0xFF, 0xFF
        ], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Bib(u16::max_value()));

        let output: Vec<u8, U2> = to_vec(&DataEnum::Bap(u8::max_value())).unwrap();
        assert_eq!(&[
            0x02,
            0xFF
        ], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Bap(u8::max_value()));

        let output: Vec<u8, U8> = to_vec(&DataEnum::Kim(EnumStruct { eight: 0xF0, sixt: 0xACAC })).unwrap();
        assert_eq!(&[
            0x03,
            0xF0,
            0xAC, 0xAC,
        ], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Kim(EnumStruct { eight: 0xF0, sixt: 0xACAC }));

        let output: Vec<u8, U8> = to_vec(&DataEnum::Chi{ a: 0x0F, b: 0xC7C7C7C7 }).unwrap();
        assert_eq!(&[
            0x04,
            0x0F,
            0xC7, 0xC7, 0xC7, 0xC7
        ], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Chi{ a: 0x0F, b: 0xC7C7C7C7 });

        let output: Vec<u8, U8> = to_vec(&DataEnum::Sho(0x6969, 0x07)).unwrap();
        assert_eq!(&[
            0x05,
            0x69, 0x69,
            0x07
        ], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Sho(0x6969, 0x07));
    }

    #[test]
    fn tuples() {
        let output: Vec<u8, U128> = to_vec(&(1u8, 10u32, "Hello!")).unwrap();
        assert_eq!(&[
            1u8,
            0x0A, 0x00, 0x00, 0x00,
            0x06,
            b'H', b'e', b'l', b'l', b'o', b'!'], output.deref());
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

    // #[derive(Serialize)]
    // struct ManyVarints {
    //     a: VarintUsize,
    //     b: VarintUsize,
    //     c: VarintUsize,
    // }

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

        // AJM: No Varints

        // let output: Vec<u8, U128> = to_vec(&ManyVarints {
        //     a: VarintUsize(0x01),
        //     b: VarintUsize(0xFFFF_FFFF),
        //     c: VarintUsize(0x07CD),
        // }).unwrap();

        // assert_eq!(&[
        //     0x01,
        //     0xFF, 0xFF, 0xFF, 0xFF, 0x0F,
        //     0xCD, 0x0F,
        // ], output.deref());
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
        }).unwrap();

        assert_eq!(&[
            0x04,
            0x01, 0x10, 0x02, 0x20,
            0x05,
            b'h', b'E', b'l', b'L', b'o',
        ], output.deref());

        let out: RefStruct = from_bytes(output.deref()).unwrap();
        assert_eq!(out, RefStruct {
            bytes: &bytes,
            str_s: message,
        });
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
        assert_eq!(&[
            0x04,
            0x01, 0x02, 0x03, 0x04
        ], output.deref());
        let out: Vec<u8, U4> = from_bytes(output.deref()).unwrap();
        assert_eq!(out, input);

        let mut input: String<U8> = String::new();
        write!(&mut input, "helLO!").unwrap();
        let output: Vec<u8, U7> = to_vec(&input).unwrap();
        assert_eq!(&[
            0x06,
            b'h', b'e', b'l', b'L', b'O', b'!'
        ], output.deref());
        let out: String<U8> = from_bytes(output.deref()).unwrap();
        assert_eq!(input, out);
    }
}
