use serde::ser::{self, Serialize, SerializeTuple, Serializer};

use crate::reserialize::{expecting, tuple::Reserializer};

pub struct SerializeTupleVariant<T>(T);
pub struct SerializeTupleStruct<T>(T);

impl<S: Serializer> Reserializer<S> for expecting::Variant<'static, expecting::data::Tuple<'_>> {
    type SerializeTuple = SerializeTupleVariant<S::SerializeTupleVariant>;

    fn reserialize_tuple(
        &self,
        serializer: S,
        len: usize,
    ) -> Result<Self::SerializeTuple, S::Error> {
        serializer
            .serialize_tuple_variant(self.enum_name, self.variant_index, self.variant_name, len)
            .map(SerializeTupleVariant)
    }
}

impl<S: Serializer> Reserializer<S> for expecting::Struct<'static, expecting::data::Tuple<'_>> {
    type SerializeTuple = SerializeTupleStruct<S::SerializeTupleStruct>;

    fn reserialize_tuple(
        &self,
        serializer: S,
        len: usize,
    ) -> Result<Self::SerializeTuple, S::Error> {
        serializer
            .serialize_tuple_struct(self.name, len)
            .map(SerializeTupleStruct)
    }
}

impl<S: ser::SerializeTupleVariant> SerializeTuple for SerializeTupleVariant<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}

impl<S: ser::SerializeTupleStruct> SerializeTuple for SerializeTupleStruct<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}
