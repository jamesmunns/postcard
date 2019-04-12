use crate::ser::flavor::SerFlavor;
use crate::error::{Error, Result};
use crate::varint::VarintUsize;
use byteorder::{ByteOrder, LittleEndian};
use serde::{ser, Serialize};

pub struct Serializer<F>
where
    F: SerFlavor,
{
    pub output: F,
}

impl<'a, F> ser::Serializer for &'a mut Serializer<F>
where
    F: SerFlavor,
{
    // The output type produced by this `Serializer` during successful
    // serializationOutput. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
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

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u8(if v { 1 } else { 0 })
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
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

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<()> {
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

    fn serialize_f32(self, v: f32) -> Result<()> {
        let mut buf = [0u8; core::mem::size_of::<f32>()];
        LittleEndian::write_f32(&mut buf, v);
        self.output
            .try_extend(&buf)
            .map_err(|_| Error::SerializeBufferFull)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        let mut buf = [0u8; core::mem::size_of::<f64>()];
        LittleEndian::write_f64(&mut buf, v);
        self.output
            .try_extend(&buf)
            .map_err(|_| Error::SerializeBufferFull)
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<()> {
        let mut buf = [0u8; 4];
        let strsl = v.encode_utf8(&mut buf);
        strsl.serialize(self)
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<()> {
        VarintUsize(v.len()).serialize(&mut *self)?;
        self.output
            .try_extend(v.as_bytes())
            .map_err(|_| Error::SerializeBufferFull)?;
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output
            .try_extend(v)
            .map_err(|_| Error::SerializeBufferFull)
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<()> {
        self.serialize_u8(0)
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_u8(1)?;
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no daunimpta, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(())
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        VarintUsize(variant_index as usize).serialize(self)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
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
        VarintUsize(variant_index as usize).serialize(&mut *self)?;
        value.serialize(self)
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        VarintUsize(len.ok_or(Error::SerializeSeqLengthUnknown)?).serialize(&mut *self)?;
        Ok(self)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        VarintUsize(variant_index as usize).serialize(&mut *self)?;
        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        // self.output += "{";
        // Ok(self)
        Err(Error::NotYetImplemented)
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        VarintUsize(variant_index as usize).serialize(&mut *self)?;
        Ok(self)
    }

    fn collect_str<T: ?Sized>(self, _value: &T) -> Result<Self::Ok>
    where
        T: core::fmt::Display,
    {
        unreachable!()
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
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

// Same thing but for tuples.
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

// Same thing but for tuple structs.
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

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
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

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a, F> ser::SerializeMap for &'a mut Serializer<F>
where
    F: SerFlavor,
{
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('{') {
        //     self.output += ",";
        // }
        // key.serialize(&mut **self)
        Err(Error::NotYetImplemented)
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // self.output += ":";
        // value.serialize(&mut **self)
        Err(Error::NotYetImplemented)

    }

    fn end(self) -> Result<()> {
        // self.output += "}";
        // Ok(())
        Err(Error::NotYetImplemented)
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
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

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
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
