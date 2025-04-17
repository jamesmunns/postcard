use core::slice;

use postcard_schema::schema::owned::OwnedNamedField;
use serde::{
    de,
    ser::{Serialize, SerializeMap, SerializeTuple, Serializer},
};

use crate::reserialize::{self, expecting};

pub struct ReserializeStructAsMap<'a> {
    pub expecting: expecting::Struct<'a, expecting::data::Struct<'a>>,
}

impl de::Expected for ReserializeStructAsMap<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        de::Expected::fmt(&self.expecting, formatter)
    }
}

impl<'a, S: Serializer> reserialize::tuple::Reserializer<S> for ReserializeStructAsMap<'a> {
    type SerializeTuple = SerializeFieldsAsMapEntries<'a, S::SerializeMap>;

    fn reserialize_tuple(
        &self,
        serializer: S,
        len: usize,
    ) -> Result<Self::SerializeTuple, <S as Serializer>::Error> {
        let serializer = serializer.serialize_map(Some(len))?;
        let fields = self.expecting.data.fields.iter();
        Ok(SerializeFieldsAsMapEntries { serializer, fields })
    }
}

pub struct SerializeFieldsAsMapEntries<'a, S> {
    serializer: S,
    fields: slice::Iter<'a, OwnedNamedField>,
}

impl<S: SerializeMap> SerializeTuple for SerializeFieldsAsMapEntries<'_, S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let field = self.fields.next().unwrap();
        self.serializer.serialize_entry(&field.name, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.end()
    }
}
