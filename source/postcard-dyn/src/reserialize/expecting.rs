use core::{
    fmt::{self, Display},
    ops::RangeTo,
};

use postcard_schema::schema::owned::OwnedVariant;
use serde::de::{self, Expected};

pub trait Unexpected: de::Error {
    fn missing_elements(len: usize, expected: &dyn Expected, expected_elements: usize) -> Self {
        Self::invalid_length(
            len,
            &super::Expected(format_args!(
                "{expected} with {expected_elements} element{}",
                if expected_elements == 1 { "" } else { "s" },
            )),
        )
    }

    fn unknown_variant_index(index: impl Into<u64>, expected: RangeTo<impl Display>) -> Self {
        Self::invalid_value(
            de::Unexpected::Unsigned(index.into()),
            &super::Expected(format_args!("variant index 0 <= i < {}", expected.end)),
        )
    }
}

impl<Error: de::Error> Unexpected for Error {}

pub struct Tuple;

pub struct Struct<'a, Data> {
    pub name: &'a str,
    pub data: Data,
}

pub struct Enum<'name, 'schema> {
    pub name: &'name str,
    pub variants: &'schema [OwnedVariant],
}

pub struct Variant<'a, Data> {
    pub enum_name: &'a str,
    pub variant_index: u32,
    pub variant_name: &'a str,
    pub data: Data,
}

pub mod data {
    use postcard_schema::schema::owned::{OwnedDataModelType, OwnedNamedField};

    pub struct Unit;
    pub struct Newtype<'a> {
        pub schema: &'a OwnedDataModelType,
    }
    pub struct Tuple<'a> {
        pub elements: &'a [OwnedDataModelType],
    }
    pub struct Struct<'a> {
        pub fields: &'a [OwnedNamedField],
    }
}

impl Expected for Tuple {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a tuple")
    }
}

impl Expected for Struct<'_, data::Unit> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "unit struct {}", self.name)
    }
}

impl Expected for Struct<'_, data::Newtype<'_>> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "tuple struct {}", self.name)
    }
}

impl Expected for Struct<'_, data::Tuple<'_>> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "tuple struct {}", self.name)
    }
}

impl Expected for Struct<'_, data::Struct<'_>> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "struct {}", self.name)
    }
}

impl Expected for Enum<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "enum {}", self.name)
    }
}

impl Expected for Variant<'_, data::Struct<'_>> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "struct variant {}::{}",
            self.enum_name, self.variant_name
        )
    }
}

impl Expected for Variant<'_, data::Tuple<'_>> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "tuple variant {}::{}",
            self.enum_name, self.variant_name
        )
    }
}
