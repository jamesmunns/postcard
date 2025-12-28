use serde_core::de::{self, DeserializeSeed, IntoDeserializer, Visitor};

use crate::de::flavors::{Flavor, Slice};
use core::marker::PhantomData;
use postcard_core::de as pcde;

/// The deserialization error type
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeserializerError<PopErr, FinErr> {
    /// A Flavor-specific error occurred while extracting data
    PopError(PopErr),
    /// A Flavor-specific error occurred while finalizing the deserialization
    FinalizeError(FinErr),
    /// A bad boolean value, e.g. NOT `0` or `1` was encountered while deserializing
    BadBool,
    /// A bad varint value was encountered while deserializing
    BadVarint,
    /// A bad [`char`] value was encounterered while deserializing
    BadChar,
    /// A bad UTF-8 string was encountered while deserializing
    BadUtf8,
    /// A bad [`Option`] was encountered while deserializing, e.g. the descriminant
    /// value was NEITHER `0` (for `None`) nor `1` (for `Some`)
    BadOption,
    /// The deserializer was requested to perform the `deserialize_any` action
    /// that postcard does not and will not ever support
    UnsupportedDeserAny,
    /// The deserializer was requested to perform the `deserialize_identifier` action
    /// that postcard does not and will not ever support
    UnsupportedDeserIdent,
    /// The deserializer was requested to perform the `deserialize_ignored_any` action
    /// that postcard does not and will not ever support
    UnsupportedDeserIgnoredAny,
    /// Serde returned a Custom error
    ///
    /// With the `alloc` or `std` features enabled, this will contain a formatted string.
    /// Without these features, the error context is not retained
    Custom(SerdeCustomError),
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
mod custom {
    #[derive(Debug, PartialEq, Eq)]
    pub struct SerdeCustomError {
        inner: (),
    }

    impl core::fmt::Display for SerdeCustomError {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("SerdeCustomError(...)")
        }
    }

    impl<PopErr, FinErr> serde_core::de::Error for super::DeserializerError<PopErr, FinErr>
    where
        PopErr: core::fmt::Debug + core::fmt::Display,
        FinErr: core::fmt::Debug + core::fmt::Display,
    {
        #[inline]
        fn custom<T>(_msg: T) -> Self
        where
            T: core::fmt::Display,
        {
            super::DeserializerError::Custom(SerdeCustomError { inner: () })
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
mod custom {
    extern crate alloc;

    #[derive(Debug, PartialEq, Eq)]
    pub struct SerdeCustomError {
        inner: alloc::string::String,
    }

    impl core::fmt::Display for SerdeCustomError {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "SerdeCustomError({})", self.inner)
        }
    }

    impl<PopErr, FinErr> serde_core::de::Error for super::DeserializerError<PopErr, FinErr>
    where
        PopErr: core::fmt::Debug + core::fmt::Display,
        FinErr: core::fmt::Debug + core::fmt::Display,
    {
        #[inline]
        fn custom<T>(msg: T) -> Self
        where
            T: core::fmt::Display,
        {
            use alloc::string::ToString;
            super::DeserializerError::Custom(SerdeCustomError {
                inner: msg.to_string(),
            })
        }
    }
}

pub use custom::SerdeCustomError;

impl<PopErr, FinErr> core::fmt::Display for DeserializerError<PopErr, FinErr>
where
    PopErr: core::fmt::Display,
    FinErr: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeserializerError::PopError(e) => write!(f, "PopError({e})"),
            DeserializerError::FinalizeError(e) => write!(f, "FinalizeError({e})"),
            DeserializerError::BadBool => f.write_str("BadBool"),
            DeserializerError::BadVarint => f.write_str("BadVarint"),
            DeserializerError::BadChar => f.write_str("BadChar"),
            DeserializerError::BadUtf8 => f.write_str("BadUtf8"),
            DeserializerError::BadOption => f.write_str("BadOption"),
            DeserializerError::UnsupportedDeserAny => f.write_str("UnsupportedDeserAny"),
            DeserializerError::UnsupportedDeserIdent => f.write_str("UnsupportedDeserIdent"),
            DeserializerError::UnsupportedDeserIgnoredAny => {
                f.write_str("UnsupportedDeserIgnoredAny")
            }
            DeserializerError::Custom(serde_custom_error) => serde_custom_error.fmt(f),
        }
    }
}

impl<PopErr, FinErr> core::error::Error for DeserializerError<PopErr, FinErr>
where
    PopErr: core::fmt::Debug + core::fmt::Display,
    FinErr: core::fmt::Debug + core::fmt::Display,
{
}

/// A `serde` compatible deserializer, generic over “Flavors” of deserializing plugins.
///
/// Please note that postcard messages are not self-describing and therefore incompatible with
/// [internally tagged enums](https://serde.rs/enum-representations.html#internally-tagged).
pub struct Deserializer<'de, F: Flavor<'de>> {
    flavor: F,
    _plt: PhantomData<&'de ()>,
}

impl<'de, F> Deserializer<'de, F>
where
    F: Flavor<'de> + 'de,
{
    /// Obtain a Deserializer from a slice of bytes
    pub fn from_flavor(flavor: F) -> Self {
        Deserializer {
            flavor,
            _plt: PhantomData,
        }
    }

    /// Return the remaining (unused) bytes in the Deserializer along with any
    /// additional data provided by the [`Flavor`]
    pub fn finalize(
        self,
    ) -> Result<F::Remainder, DeserializerError<F::PopError, F::FinalizeError>> {
        self.flavor
            .finalize()
            .map_err(DeserializerError::FinalizeError)
    }
}

impl<'de> Deserializer<'de, Slice<'de>> {
    /// Obtain a Deserializer from a slice of bytes
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            flavor: Slice::new(input),
            _plt: PhantomData,
        }
    }
}

struct SeqAccess<'a, 'b, F: Flavor<'b>> {
    deserializer: &'a mut Deserializer<'b, F>,
    len: usize,
}

impl<'a, 'b: 'a, F: Flavor<'b>> serde_core::de::SeqAccess<'b> for SeqAccess<'a, 'b, F> {
    type Error = DeserializerError<F::PopError, F::FinalizeError>;

    #[inline]
    fn next_element_seed<V: DeserializeSeed<'b>>(
        &mut self,
        seed: V,
    ) -> Result<Option<V::Value>, DeserializerError<F::PopError, F::FinalizeError>> {
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

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        match self.deserializer.flavor.size_hint() {
            Some(size) if size < self.len => None,
            _ => Some(self.len),
        }
    }
}

struct MapAccess<'a, 'b, F: Flavor<'b>> {
    deserializer: &'a mut Deserializer<'b, F>,
    len: usize,
}

impl<'a, 'b: 'a, F: Flavor<'b>> serde_core::de::MapAccess<'b> for MapAccess<'a, 'b, F> {
    type Error = DeserializerError<F::PopError, F::FinalizeError>;

    #[inline]
    fn next_key_seed<K: DeserializeSeed<'b>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, DeserializerError<F::PopError, F::FinalizeError>> {
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

    #[inline]
    fn next_value_seed<V: DeserializeSeed<'b>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>> {
        DeserializeSeed::deserialize(seed, &mut *self.deserializer)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'de, F: Flavor<'de>> de::Deserializer<'de> for &mut Deserializer<'de, F> {
    type Error = DeserializerError<F::PopError, F::FinalizeError>;

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    // Postcard does not support structures not known at compile time
    #[inline]
    fn deserialize_any<V>(
        self,
        _visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        // We wont ever support this.
        Err(DeserializerError::UnsupportedDeserAny)
    }

    // Take a boolean encoded as a u8
    #[inline]
    fn deserialize_bool<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let val = match pcde::try_take_bool(&mut self.flavor) {
            Ok(Some(b)) => b,
            Ok(None) => return Err(DeserializerError::BadBool),
            Err(e) => return Err(DeserializerError::PopError(e)),
        };
        visitor.visit_bool(val)
    }

    #[inline]
    fn deserialize_i8<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.flavor.pop().map_err(DeserializerError::PopError)? as i8)
    }

    #[inline]
    fn deserialize_i16<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let v = pcde::try_take_i16(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let v = v.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_i16(v)
    }

    #[inline]
    fn deserialize_i32<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let v = pcde::try_take_i32(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let v = v.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_i32(v)
    }

    #[inline]
    fn deserialize_i64<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let v = pcde::try_take_i64(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let v = v.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_i64(v)
    }

    #[inline]
    fn deserialize_i128<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let v = pcde::try_take_i128(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let v = v.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_i128(v)
    }

    #[inline]
    fn deserialize_u8<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let v = pcde::try_take_u8(&mut self.flavor).map_err(DeserializerError::PopError)?;
        visitor.visit_u8(v)
    }

    #[inline]
    fn deserialize_u16<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let v = pcde::try_take_u16(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let v = v.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_u16(v)
    }

    #[inline]
    fn deserialize_u32<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let v = pcde::try_take_u32(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let v = v.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_u32(v)
    }

    #[inline]
    fn deserialize_u64<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let v = pcde::try_take_u64(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let v = v.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_u64(v)
    }

    #[inline]
    fn deserialize_u128<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let v = pcde::try_take_u128(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let v = v.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_u128(v)
    }

    #[inline]
    fn deserialize_f32<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let f = pcde::try_take_f32(&mut self.flavor).map_err(DeserializerError::PopError)?;
        visitor.visit_f32(f)
    }

    #[inline]
    fn deserialize_f64<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let f = pcde::try_take_f64(&mut self.flavor).map_err(DeserializerError::PopError)?;
        visitor.visit_f64(f)
    }

    #[inline]
    fn deserialize_char<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let ch = pcde::try_take_char(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let ch = ch.ok_or(DeserializerError::BadUtf8)?;
        visitor.visit_char(ch)
    }

    #[inline]
    fn deserialize_str<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let str_sl = pcde::try_take_str(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let str_sl = str_sl.ok_or(DeserializerError::BadUtf8)?;
        visitor.visit_borrowed_str(str_sl)
    }

    #[inline]
    fn deserialize_string<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let str_sl =
            pcde::try_take_str_temp(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let str_sl = str_sl.ok_or(DeserializerError::BadUtf8)?;
        visitor.visit_str(str_sl)
    }

    #[inline]
    fn deserialize_bytes<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let bytes = pcde::try_take_bytes(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let bytes = bytes.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_borrowed_bytes(bytes)
    }

    #[inline]
    fn deserialize_byte_buf<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let bytes =
            pcde::try_take_bytes_temp(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let bytes = bytes.ok_or(DeserializerError::BadVarint)?;
        visitor.visit_bytes(bytes)
    }

    #[inline]
    fn deserialize_option<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        match pcde::try_take_option_discrim(&mut self.flavor) {
            Ok(Some(false)) => visitor.visit_none(),
            Ok(Some(true)) => visitor.visit_some(self),
            Ok(None) => Err(DeserializerError::BadOption),
            Err(e) => Err(DeserializerError::PopError(e)),
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    // Unit is not actually encoded in Postcard.
    #[inline]
    fn deserialize_unit<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    // Unit struct means a named value containing no data.
    // Unit structs are not actually encoded in Postcard.
    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_seq<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let len = pcde::try_take_length(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let len = len.ok_or(DeserializerError::BadVarint)?;

        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len,
        })
    }

    #[inline]
    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len,
        })
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_map<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        let len = pcde::try_take_length(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let len = len.ok_or(DeserializerError::BadVarint)?;

        visitor.visit_map(MapAccess {
            deserializer: self,
            len,
        })
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    // As a binary format, Postcard does not encode identifiers
    #[inline]
    fn deserialize_identifier<V>(
        self,
        _visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        // Will not support
        Err(DeserializerError::UnsupportedDeserIdent)
    }

    #[inline]
    fn deserialize_ignored_any<V>(
        self,
        _visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>>
    where
        V: Visitor<'de>,
    {
        // Will not support
        Err(DeserializerError::UnsupportedDeserIgnoredAny)
    }
}

impl<'de, F: Flavor<'de>> serde_core::de::VariantAccess<'de> for &mut Deserializer<'de, F> {
    type Error = DeserializerError<F::PopError, F::FinalizeError>;

    #[inline]
    fn unit_variant(self) -> Result<(), DeserializerError<F::PopError, F::FinalizeError>> {
        Ok(())
    }

    #[inline]
    fn newtype_variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>> {
        DeserializeSeed::deserialize(seed, self)
    }

    #[inline]
    fn tuple_variant<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>> {
        serde_core::de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    #[inline]
    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeserializerError<F::PopError, F::FinalizeError>> {
        serde_core::de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

impl<'de, F: Flavor<'de>> serde_core::de::EnumAccess<'de> for &mut Deserializer<'de, F> {
    type Error = DeserializerError<F::PopError, F::FinalizeError>;
    type Variant = Self;

    #[inline]
    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self), DeserializerError<F::PopError, F::FinalizeError>> {
        let varint =
            pcde::try_take_discriminant(&mut self.flavor).map_err(DeserializerError::PopError)?;
        let varint = varint.ok_or(DeserializerError::BadVarint)?;
        let v = DeserializeSeed::deserialize(seed, varint.into_deserializer())?;
        Ok((v, self))
    }
}
