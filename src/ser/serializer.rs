use serde::{ser, Serialize};

use crate::error::{Error, Result};
use crate::ser::flavors::SerFlavor;
use crate::varint::VarintUsize;

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
    F: SerFlavor,
{
    /// This is the Flavor(s) that will be used to modify or store any bytes generated
    /// by serialization
    pub output: F,
}

impl<'a, F> ser::Serializer for &'a mut Serializer<F>
where
    F: SerFlavor,
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

    fn is_human_readable(&self) -> bool {
        false
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u8(if v { 1 } else { 0 })
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_u8(v.to_le_bytes()[0])
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.output
            .try_extend(&v.to_le_bytes())
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.output
            .try_extend(&v.to_le_bytes())
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output
            .try_extend(&v.to_le_bytes())
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.output
            .try_extend(&v.to_le_bytes())
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output
            .try_push(v)
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output
            .try_extend(&v.to_le_bytes())
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output
            .try_extend(&v.to_le_bytes())
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output
            .try_extend(&v.to_le_bytes())
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.output
            .try_extend(&v.to_le_bytes())
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        let buf = v.to_bits().to_le_bytes();
        self.output
            .try_extend(&buf)
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        let buf = v.to_bits().to_le_bytes();
        self.output
            .try_extend(&buf)
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let mut buf = [0u8; 4];
        let strsl = v.encode_utf8(&mut buf);
        strsl.serialize(self)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output
            .try_push_varint_usize(&VarintUsize(v.len()))
            .map_err(|_| Error::SerializeBufferFull)?;
        self.output
            .try_extend(v.as_bytes())
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output
            .try_push_varint_usize(&VarintUsize(v.len()))
            .map_err(|_| Error::SerializeBufferFull)?;
        self.output
            .try_extend(v)
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_u8(0)
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_u8(1)?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.output
            .try_push_varint_usize(&VarintUsize(variant_index as usize))
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

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
            .try_push_varint_usize(&VarintUsize(variant_index as usize))
            .map_err(|_| Error::SerializeBufferFull)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output
            .try_push_varint_usize(&VarintUsize(len.ok_or(Error::SerializeSeqLengthUnknown)?))
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output
            .try_push_varint_usize(&VarintUsize(variant_index as usize))
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output
            .try_push_varint_usize(&VarintUsize(len.ok_or(Error::SerializeSeqLengthUnknown)?))
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output
            .try_push_varint_usize(&VarintUsize(variant_index as usize))
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(self)
    }

    fn collect_str<T: ?Sized>(self, _value: &T) -> Result<Self::Ok>
    where
        T: core::fmt::Display,
    {
        unreachable!()
    }
}

impl<'a, F> ser::SerializeSeq for &'a mut Serializer<F>
where
    F: SerFlavor,
{
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeTuple for &'a mut Serializer<F>
where
    F: SerFlavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeTupleStruct for &'a mut Serializer<F>
where
    F: SerFlavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeTupleVariant for &'a mut Serializer<F>
where
    F: SerFlavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeMap for &'a mut Serializer<F>
where
    F: SerFlavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeStruct for &'a mut Serializer<F>
where
    F: SerFlavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeStructVariant for &'a mut Serializer<F>
where
    F: SerFlavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
