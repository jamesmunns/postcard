//! Lossless reserialization.
//!
//! As noted [above](super), lossless serialization is only possible by compromising elsewhere.
//! This module provides implementations with different compromises:
//! - [`reserialize_leaky()`] leaks memory for each unique struct/enum/variant/field name

use core::{cell::RefCell, fmt, str};

use postcard_schema::schema::owned::OwnedDataModelType;
use serde::{
    de::{self, Deserializer},
    ser::Serializer,
};

use crate::Error;

use super::{
    expecting,
    strategy::{self, Strategy as _},
    Context,
};

mod interned;
use interned::Interned;

mod enums;
mod structs;
mod tuples;

/// Reserialize [`postcard`]-encoded data losslessly, **leaking memory**.
///
/// In order to serialize structs and enums losslessly, this **allocates and leaks each unique
/// struct/enum/variant/field name, and the list of field names for each struct**.
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
pub fn reserialize_leaky<'de, D, S>(
    schema: &OwnedDataModelType,
    deserializer: D,
    serializer: S,
) -> Result<S::Ok, Error<D::Error, S::Error>>
where
    D: Deserializer<'de>,
    S: Serializer,
{
    Strategy.reserialize(schema, deserializer, serializer)
}

/// Reserialize structs and enums losslessly, **leaking memory**.
struct Strategy;

impl Strategy {
    fn with_interned<T>(&self, f: impl FnOnce(&mut Interned) -> T) -> T {
        thread_local! {
            static INTERNED: RefCell<Interned> = RefCell::new(Default::default());
        }
        INTERNED.with_borrow_mut(f)
    }
}

impl strategy::Strategy for Strategy {
    fn reserialize_unit_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        name: &str,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        struct Visitor<S> {
            serializer: S,
            expecting: expecting::Struct<'static, expecting::data::Unit>,
        }

        impl<S: Serializer> de::Visitor<'_> for Visitor<S> {
            type Value = Result<S::Ok, S::Error>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                de::Expected::fmt(&self.expecting, formatter)
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(self.serializer.serialize_unit_struct(self.expecting.name))
            }
        }

        let name = context
            .strategy
            .with_interned(|interned| interned.intern_identifier(name));
        deserializer.deserialize_unit_struct(
            name,
            Visitor {
                serializer,
                expecting: expecting::Struct {
                    name,
                    data: expecting::data::Unit,
                },
            },
        )
    }

    fn reserialize_newtype_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Struct<'_, expecting::data::Newtype>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        struct Visitor<'a, S> {
            context: &'a Context<'a, Strategy>,
            serializer: S,
            expecting: expecting::Struct<'static, expecting::data::Newtype<'a>>,
        }

        impl<'de, S: Serializer> de::Visitor<'de> for Visitor<'_, S> {
            type Value = Result<S::Ok, S::Error>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                de::Expected::fmt(&self.expecting, formatter)
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                self.context
                    .reserialize_ty(self.expecting.data.schema, deserializer, |inner| {
                        self.serializer
                            .serialize_newtype_struct(self.expecting.name, inner)
                    })
            }
        }

        let name = context
            .strategy
            .with_interned(|interned| interned.intern_identifier(expecting.name));
        deserializer.deserialize_newtype_struct(
            name,
            Visitor {
                context,
                serializer,
                expecting: expecting::Struct {
                    name,
                    data: expecting.data,
                },
            },
        )
    }

    fn reserialize_tuple_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Struct<'_, expecting::data::Tuple>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        let name = context
            .strategy
            .with_interned(|interned| interned.intern_identifier(expecting.name));
        deserializer.deserialize_tuple_struct(
            name,
            expecting.data.elements.len(),
            super::tuple::Visitor {
                context,
                serializer,
                fields: expecting.data.elements,
                reserializer: expecting::Struct {
                    name,
                    data: expecting.data,
                },
            },
        )
    }

    fn reserialize_struct<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Struct<'_, expecting::data::Struct>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        let fields = expecting.data.fields;
        let (name, field_names) = context.strategy.with_interned(|interned| {
            let name = interned.intern_identifier(expecting.name);
            let field_names = interned.intern_slice(fields.iter().map(|f| f.name.as_ref()));
            (name, field_names)
        });
        deserializer.deserialize_struct(
            name,
            field_names,
            structs::Visitor {
                context,
                serializer,
                fields,
                field_names,
                reserializer: expecting::Struct {
                    name,
                    data: expecting.data,
                },
            },
        )
    }

    fn reserialize_enum<'de, D: Deserializer<'de>, S: Serializer>(
        context: &Context<'_, Self>,
        deserializer: D,
        serializer: S,
        expecting: expecting::Enum<'_, '_>,
    ) -> Result<Result<S::Ok, S::Error>, D::Error> {
        let variants = expecting.variants;
        let (name, variant_names) = context.strategy.with_interned(|interned| {
            let name = interned.intern_identifier(expecting.name);
            let variant_names = interned.intern_slice(variants.iter().map(|v| v.name.as_ref()));
            (name, variant_names)
        });
        deserializer.deserialize_enum(
            name,
            variant_names,
            enums::Visitor {
                context,
                serializer,
                expecting: expecting::Enum { name, variants },
                variant_names,
            },
        )
    }
}
