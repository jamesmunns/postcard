//! Lossless reserialization.
//!
//! As noted [above](super), lossless serialization is only possible by compromising elsewhere.
//! This module provides implementations with different compromises:
//! - [`reserialize_leaky()`] leaks memory for each unique struct/enum/variant/field name

use core::cell::RefCell;
use std::collections::HashSet;

use postcard::de_flavors::Flavor;
use postcard_schema::schema::owned::{
    OwnedDataModelVariant, OwnedNamedType, OwnedNamedValue, OwnedNamedVariant,
};
use serde::{
    ser::{SerializeStruct, SerializeStructVariant, SerializeTupleStruct, SerializeTupleVariant},
    Serialize, Serializer,
};

use crate::Error;

use super::{reserialize, structs_and_enums::Reserialize};

/// Reserialize [`postcard`]-encoded data losslessly, **leaking memory**.
///
/// In order to serialize structs and enums losslessly, this **allocates and leaks each unique
/// struct/enum/variant/field name**.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// # use postcard::ser_flavors::Flavor;
/// use postcard_schema::Schema;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Schema, PartialEq, Debug)]
/// enum Foo {
///     Bar { a: u8, b: u8 },
/// }
///
/// let value = Foo::Bar { a: 5, b: 10 };
/// let bytes = postcard::to_allocvec(&value)?;
/// let mut serializer = postcard::Serializer {
///     output: postcard::ser_flavors::StdVec::new(),
/// };
/// postcard_dyn::reserialize::lossless::reserialize_leaky(
///     &Foo::SCHEMA.into(),
///     &mut postcard::Deserializer::from_bytes(&bytes),
///     &mut serializer,
/// )?;
/// let out = serializer.output.finalize()?;
/// let deserialized: Foo = postcard::from_bytes(&out)?;
/// assert_eq!(deserialized, value);
/// # Ok(())
/// # }
/// ```
pub fn reserialize_leaky<'de, F, S>(
    schema: &OwnedNamedType,
    deserializer: &mut postcard::Deserializer<'de, F>,
    serializer: S,
) -> Result<S::Ok, Error<S::Error>>
where
    F: Flavor<'de>,
    S: Serializer,
{
    reserialize(schema, deserializer, serializer, Strategy)
}

/// Reserialize structs and enums losslessly, **leaking memory**.
struct Strategy;

impl Strategy {
    fn intern(&self, s: &str) -> &'static str {
        thread_local! {
            static STRINGS: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
        }
        STRINGS.with_borrow_mut(|strings| {
            if !strings.contains(s) {
                strings.insert(String::leak(s.to_string()));
            }
            *strings.get(s).unwrap()
        })
    }
}

impl super::structs_and_enums::Strategy for Strategy {
    fn serialize_unit_struct<S: Serializer>(
        &self,
        serializer: S,
        name: &str,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit_struct(self.intern(name))
    }

    fn serialize_newtype_struct<S: Serializer, T: ?Sized + Serialize>(
        &self,
        serializer: S,
        name: &str,
        value: &T,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct(self.intern(name), value)
    }

    fn serialize_tuple_struct<S: Serializer>(
        &self,
        serializer: S,
        reserialize: impl Reserialize,
        name: &str,
        fields: &[OwnedNamedType],
    ) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_tuple_struct(self.intern(name), fields.len())?;
        for field in fields {
            serializer.serialize_field(&reserialize.with_schema(field))?;
        }
        serializer.end()
    }

    fn serialize_struct<S: Serializer>(
        &self,
        serializer: S,
        reserialize: impl Reserialize,
        name: &str,
        fields: &[OwnedNamedValue],
    ) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_struct(self.intern(name), fields.len())?;
        for field in fields {
            serializer.serialize_field(
                self.intern(&field.name),
                &reserialize.with_schema(&field.ty),
            )?;
        }
        serializer.end()
    }

    fn serialize_enum<S: Serializer>(
        &self,
        serializer: S,
        reserialize: impl Reserialize,
        name: &str,
        variant_index: u32,
        variant: &OwnedNamedVariant,
    ) -> Result<S::Ok, S::Error> {
        match &variant.ty {
            OwnedDataModelVariant::UnitVariant => serializer.serialize_unit_variant(
                self.intern(name),
                variant_index,
                self.intern(&variant.name),
            ),
            OwnedDataModelVariant::NewtypeVariant(inner) => serializer.serialize_newtype_variant(
                self.intern(name),
                variant_index,
                self.intern(&variant.name),
                &reserialize.with_schema(inner),
            ),
            OwnedDataModelVariant::TupleVariant(fields) => {
                let mut serializer = serializer.serialize_tuple_variant(
                    self.intern(name),
                    variant_index,
                    self.intern(&variant.name),
                    fields.len(),
                )?;
                for field in fields {
                    serializer.serialize_field(&reserialize.with_schema(field))?;
                }
                serializer.end()
            }
            OwnedDataModelVariant::StructVariant(fields) => {
                let mut serializer = serializer.serialize_struct_variant(
                    self.intern(name),
                    variant_index,
                    self.intern(&variant.name),
                    fields.len(),
                )?;
                for field in fields {
                    serializer.serialize_field(
                        self.intern(&field.name),
                        &reserialize.with_schema(&field.ty),
                    )?;
                }
                serializer.end()
            }
        }
    }
}
