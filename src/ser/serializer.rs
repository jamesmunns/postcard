use serde::{ser, Serialize};

use crate::error::{Error, Result};
use crate::ser::flavors::{Flavor, Encoder};

/// A `serde` compatible serializer, generic over "Flavors" of serializing plugins.
///
/// It should rarely be necessary to directly use this type unless you are implementing your
/// own [`SerFlavor`].
///
/// See the docs for [`SerFlavor`] for more information about "flavors" of serialization
///
/// [`SerFlavor`]: trait.SerFlavor.html
pub struct Serializer<F>
where
    F: Flavor,
{
    /// This is the Flavor(s) that will be used to modify or store any bytes generated
    /// by serialization
    pub output: Encoder<F>,
}

impl<'a, F> ser::Serializer for &'a mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();

    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u8(if v { 1 } else { 0 })
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_u8(v.to_le_bytes()[0])
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<()> {
        let zzv = zig_zag_i16(v);
        self.output
            .try_push_varint_u16(zzv)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<()> {
        let zzv = zig_zag_i32(v);
        self.output
            .try_push_varint_u32(zzv)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<()> {
        let zzv = zig_zag_i64(v);
        self.output
            .try_push_varint_u64(zzv)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<()> {
        let zzv = zig_zag_i128(v);
        self.output
            .try_push_varint_u128(zzv)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output
            .flavor
            .try_push(v)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output
            .try_push_varint_u16(v)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output
            .try_push_varint_u32(v)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output
            .try_push_varint_u64(v)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<()> {
        self.output
            .try_push_varint_u128(v)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<()> {
        let buf = v.to_bits().to_le_bytes();
        self.output
            .flavor
            .try_extend(&buf)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<()> {
        let buf = v.to_bits().to_le_bytes();
        self.output
            .flavor
            .try_extend(&buf)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<()> {
        let mut buf = [0u8; 4];
        let strsl = v.encode_utf8(&mut buf);
        strsl.serialize(self)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<()> {
        self.output
            .try_push_varint_usize(v.len())
            .map_err(|_| Error::SerializeBufferFull)?;
        self.output
            .flavor
            .try_extend(v.as_bytes())
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output
            .try_push_varint_usize(v.len())
            .map_err(|_| Error::SerializeBufferFull)?;
        self.output
            .flavor
            .try_extend(v)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.serialize_u8(0)
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_u8(1)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.output
            .try_push_varint_u32(variant_index)
            .map_err(|_| Error::SerializeBufferFull)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output
            .try_push_varint_u32(variant_index)
            .map_err(|_| Error::SerializeBufferFull)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output
            .try_push_varint_usize(len.ok_or(Error::SerializeSeqLengthUnknown)?)
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output
            .try_push_varint_u32(variant_index)
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(self)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output
            .try_push_varint_usize(len.ok_or(Error::SerializeSeqLengthUnknown)?)
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(self)
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output
            .try_push_varint_u32(variant_index)
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(self)
    }

    #[inline]
    fn collect_str<T: ?Sized>(self, _value: &T) -> Result<Self::Ok>
    where
        T: core::fmt::Display,
    {
        unreachable!()
    }
}

impl<'a, F> ser::SerializeSeq for &'a mut Serializer<F>
where
    F: Flavor,
{
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeTuple for &'a mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeTupleStruct for &'a mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeTupleVariant for &'a mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeMap for &'a mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeStruct for &'a mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeStructVariant for &'a mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

fn zig_zag_i16(n: i16) -> u16 {
    ((n << 1) ^ (n >> 15)) as u16
}

fn zig_zag_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

fn zig_zag_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

fn zig_zag_i128(n: i128) -> u128 {
    ((n << 1) ^ (n >> 127)) as u128
}
