use serde_core::{Serialize, ser};

use crate::ser::flavors::Flavor;
use postcard_core::ser as pcser;

/// A `serde` compatible serializer, generic over "Flavors" of serializing plugins.
///
/// It should rarely be necessary to directly use this type unless you are implementing your
/// own [`SerFlavor`].
///
/// See the docs for [`SerFlavor`] for more information about "flavors" of serialization
///
/// [`SerFlavor`]: crate::ser_flavors::Flavor
pub struct Serializer<F>
where
    F: Flavor,
{
    /// This is the Flavor(s) that will be used to modify or store any bytes generated
    /// by serialization
    pub output: F,
}

/// The serialization error type
#[derive(Debug)]
#[non_exhaustive]
pub enum SerializerError<PushErr, FinErr> {
    /// A Flavor-specific error occurred while inserting data
    PushError(PushErr),
    /// A Flavor-specific error occurred while finalizing
    FinalizeError(FinErr),
    /// A `Seq` or `Map` was attempted to be serialized with no length
    /// hint, e.g. `Serializer::serialize_seq(None)` or
    /// `Serializer::serialize_map(None)`. This is unsupported in postcard.
    SeqLengthUnknown,
    /// Serde returned a Custom error
    ///
    /// With the `alloc` or `std` features enabled, this will contain a formatted string.
    /// Without these features, the error context is not retained
    Custom(SerdeCustomError),
}

impl<PushErr, FinErr> core::fmt::Display for SerializerError<PushErr, FinErr>
where
    PushErr: core::fmt::Display,
    FinErr: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SerializerError::PushError(e) => write!(f, "PushError({})", e),
            SerializerError::FinalizeError(e) => write!(f, "FinalizeError({})", e),
            SerializerError::SeqLengthUnknown => f.write_str("SeqLengthUnknown"),
            SerializerError::Custom(serde_custom_error) => serde_custom_error.fmt(f),
        }
    }
}

impl<PushErr, FinErr> core::error::Error for SerializerError<PushErr, FinErr>
where
    PushErr: core::fmt::Debug + core::fmt::Display,
    FinErr: core::fmt::Debug + core::fmt::Display,
{
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

    impl<PopErr, FinErr> serde_core::ser::Error for super::SerializerError<PopErr, FinErr>
    where
        PopErr: core::fmt::Debug + core::fmt::Display,
        FinErr: core::fmt::Debug + core::fmt::Display,
    {
        #[inline]
        fn custom<T>(_msg: T) -> Self
        where
            T: core::fmt::Display,
        {
            super::SerializerError::Custom(SerdeCustomError { inner: () })
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

    impl<PopErr, FinErr> serde_core::ser::Error for super::SerializerError<PopErr, FinErr>
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
            super::SerializerError::Custom(SerdeCustomError {
                inner: msg.to_string(),
            })
        }
    }
}

pub use custom::SerdeCustomError;

impl<F> ser::Serializer for &mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();

    type Error = SerializerError<F::PushError, F::FinalizeError>;

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
    fn serialize_bool(
        self,
        v: bool,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_bool(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_i8(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_i16(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_i32(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_i64(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_i128(
        self,
        v: i128,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_i128(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_u8(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_u16(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_u32(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_u64(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_u128(
        self,
        v: u128,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_u128(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_f32(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_f64(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_char(
        self,
        v: char,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        let mut buf = [0u8; 4];
        let strsl = v.encode_utf8(&mut buf);
        strsl.serialize(self)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_str(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_bytes(
        self,
        v: &[u8],
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_bytes(&mut self.output, v).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_none(self) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_option_none(&mut self.output).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_some<T>(
        self,
        value: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        pcser::try_push_option_some(&mut self.output).map_err(SerializerError::PushError)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_u32(&mut self.output, variant_index).map_err(SerializerError::PushError)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
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
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        pcser::try_push_u32(&mut self.output, variant_index).map_err(SerializerError::PushError)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> Result<Self::SerializeSeq, SerializerError<F::PushError, F::FinalizeError>> {
        let len = len.ok_or(SerializerError::SeqLengthUnknown)?;
        pcser::try_push_usize(&mut self.output, len).map_err(SerializerError::PushError)?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(
        self,
        _len: usize,
    ) -> Result<Self::SerializeTuple, SerializerError<F::PushError, F::FinalizeError>> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerializerError<F::PushError, F::FinalizeError>> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_u32(&mut self.output, variant_index).map_err(SerializerError::PushError)?;
        Ok(self)
    }

    #[inline]
    fn serialize_map(
        self,
        len: Option<usize>,
    ) -> Result<Self::SerializeMap, SerializerError<F::PushError, F::FinalizeError>> {
        let len = len.ok_or(SerializerError::SeqLengthUnknown)?;
        pcser::try_push_usize(&mut self.output, len).map_err(SerializerError::PushError)?;
        Ok(self)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, SerializerError<F::PushError, F::FinalizeError>> {
        Ok(self)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, SerializerError<F::PushError, F::FinalizeError>> {
        pcser::try_push_u32(&mut self.output, variant_index).map_err(SerializerError::PushError)?;
        Ok(self)
    }

    #[inline]
    fn collect_str<T>(
        self,
        value: &T,
    ) -> Result<Self::Ok, SerializerError<F::PushError, F::FinalizeError>>
    where
        T: core::fmt::Display + ?Sized,
    {
        use core::fmt::Write;

        // Unfortunately, we need to know the size of the serialized data before
        // we can place it into the output. In order to do this, we run the formatting
        // of the output data TWICE, the first time to determine the length, the
        // second time to actually format the data
        //
        // There are potentially other ways to do this, such as:
        //
        // * Reserving a fixed max size, such as 5 bytes, for the length field, and
        //     leaving non-canonical trailing zeroes at the end. This would work up
        //     to some reasonable length, but might have some portability vs max size
        //     tradeoffs, e.g. 64KiB if we pick 3 bytes, or 4GiB if we pick 5 bytes
        // * Expose some kind of "memmove" capability to flavors, to allow us to
        //     format into the buffer, then "scoot over" that many times.
        //
        // Despite the current approaches downside in speed, it is likely flexible
        // enough for the rare-ish case where formatting a Debug impl is necessary.
        // This is better than the previous panicking behavior, and can be improved
        // in the future.
        struct CountWriter {
            ct: usize,
        }
        impl Write for CountWriter {
            fn write_str(&mut self, s: &str) -> core::result::Result<(), core::fmt::Error> {
                self.ct += s.len();
                Ok(())
            }
        }

        let mut ctr = CountWriter { ct: 0 };

        // This is the first pass through, where we just count the length of the
        // data that we are given. The count writer cannot fail (unless value expands
        // to more than `usize` bytes).
        let _ = write!(&mut ctr, "{value}");
        let len = ctr.ct;
        pcser::try_push_usize(&mut self.output, len).map_err(SerializerError::PushError)?;

        struct FmtWriter<'a, IF>
        where
            IF: Flavor,
        {
            output: &'a mut IF,
            err: Option<IF::PushError>,
        }
        impl<IF> Write for FmtWriter<'_, IF>
        where
            IF: Flavor,
        {
            fn write_str(&mut self, s: &str) -> core::result::Result<(), core::fmt::Error> {
                match self.output.try_extend(s.as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        self.err = Some(e);
                        Err(core::fmt::Error)
                    }
                }
            }
        }

        // This second pass actually inserts the data.
        let mut fw = FmtWriter {
            output: &mut self.output,
            err: None,
        };
        let _ = write!(&mut fw, "{value}");

        if let Some(e) = fw.err {
            Err(SerializerError::PushError(e))
        } else {
            Ok(())
        }
    }
}

impl<F> ser::SerializeSeq for &mut Serializer<F>
where
    F: Flavor,
{
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = SerializerError<F::PushError, F::FinalizeError>;

    // Serialize a single element of the sequence.
    #[inline]
    fn serialize_element<T>(
        &mut self,
        value: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    #[inline]
    fn end(self) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        Ok(())
    }
}

impl<F> ser::SerializeTuple for &mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = SerializerError<F::PushError, F::FinalizeError>;

    #[inline]
    fn serialize_element<T>(
        &mut self,
        value: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        Ok(())
    }
}

impl<F> ser::SerializeTupleStruct for &mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = SerializerError<F::PushError, F::FinalizeError>;

    #[inline]
    fn serialize_field<T>(
        &mut self,
        value: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        Ok(())
    }
}

impl<F> ser::SerializeTupleVariant for &mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = SerializerError<F::PushError, F::FinalizeError>;

    #[inline]
    fn serialize_field<T>(
        &mut self,
        value: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        Ok(())
    }
}

impl<F> ser::SerializeMap for &mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = SerializerError<F::PushError, F::FinalizeError>;

    #[inline]
    fn serialize_key<T>(
        &mut self,
        key: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    #[inline]
    fn serialize_value<T>(
        &mut self,
        value: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        Ok(())
    }
}

impl<F> ser::SerializeStruct for &mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = SerializerError<F::PushError, F::FinalizeError>;

    #[inline]
    fn serialize_field<T>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        Ok(())
    }
}

impl<F> ser::SerializeStructVariant for &mut Serializer<F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = SerializerError<F::PushError, F::FinalizeError>;

    #[inline]
    fn serialize_field<T>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), SerializerError<F::PushError, F::FinalizeError>>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<(), SerializerError<F::PushError, F::FinalizeError>> {
        Ok(())
    }
}
