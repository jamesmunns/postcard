//! Flavor traits and error types for deserialization

/// Unexpectedly reached the end of the deserialization buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt-v1", derive(defmt::Format))]
pub struct UnexpectedEnd;

impl core::fmt::Display for UnexpectedEnd {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("UnexpectedEnd")
    }
}

impl core::error::Error for UnexpectedEnd {}

/// An error while decoding a postcard item
// postcard[impl conformance.decode.reject]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt-v1", derive(defmt::Format))]
#[non_exhaustive]
pub enum DecodeError<E> {
    /// Error popping bytes, likely we have run out of input
    Pop(E),
    /// The decoded varint exceeded the capacity or maximum encoded length of the
    /// requested type
    BadVarint,
    /// Observed a [bool] that was neither `0` nor `1`
    BadBool,
    /// Observed an [`Option`] discriminant that was neither `0` (`None`) nor `1` (`Some`)
    BadOption,
    /// Observed a string that was not valid UTF-8
    BadUtf8,
    /// Observed a [char] that was not exactly one UTF-8 encoded character
    BadChar,
}

impl<E> core::fmt::Display for DecodeError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::Pop(_) => f.write_str("Pop"),
            DecodeError::BadVarint => f.write_str("BadVarint"),
            DecodeError::BadBool => f.write_str("BadBool"),
            DecodeError::BadOption => f.write_str("BadOption"),
            DecodeError::BadUtf8 => f.write_str("BadUtf8"),
            DecodeError::BadChar => f.write_str("BadChar"),
        }
    }
}

impl<E: core::error::Error + 'static> core::error::Error for DecodeError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            DecodeError::Pop(e) => Some(e),
            _ => None,
        }
    }
}

/// The deserialization Flavor trait
///
/// This is used as the primary way to decode serialized data from some kind of buffer,
/// or modify that data in a middleware style pattern.
///
/// See the module level docs for an example of how flavors are used.
pub trait Flavor<'de> {
    /// The remaining data of this flavor after deserializing has completed.
    ///
    /// Typically, this includes the remaining buffer that was not used for
    /// deserialization, and in cases of more complex flavors, any additional
    /// information that was decoded or otherwise calculated during
    /// the deserialization process.
    type Remainder;

    /// The error type specific to pushing methods.
    ///
    /// This includes [`Self::pop`], [`Self::try_take_n`], and [`Self::try_take_n_temp`].
    ///
    /// If the only error is "no more data", consider using [`UnexpectedEnd`].
    type PopError: core::error::Error + 'static;

    /// The error type specific to [`Self::finalize`].
    ///
    /// If this type cannot error when pushing, e.g. for storage flavors that don't
    /// perform any meaningful finalization actions, consider using
    /// [`Infallible`](core::convert::Infallible).
    type FinalizeError: core::error::Error + 'static;

    /// Obtain the next byte for deserialization
    fn pop(&mut self) -> Result<u8, Self::PopError>;

    /// Returns the number of bytes remaining in the message, if known.
    ///
    /// # Implementation notes
    ///
    /// It is not enforced that this number is exactly correct.
    /// A flavor may yield less or more bytes than the what is hinted at by
    /// this function.
    ///
    /// `size_hint()` is primarily intended to be used for optimizations such as
    /// reserving space for deserialized items, but must not be trusted to
    /// e.g., omit bounds checks in unsafe code. An incorrect implementation of
    /// `size_hint()` should not lead to memory safety violations.
    ///
    /// That said, the implementation should provide a correct estimation,
    /// because otherwise it would be a violation of the trait’s protocol.
    ///
    /// The default implementation returns `None` which is correct for any flavor.
    fn size_hint(&self) -> Option<usize> {
        None
    }

    /// Attempt to take the next `ct` bytes from the serialized message.
    ///
    /// This variant borrows the data from the input for zero-copy deserialization. If zero-copy
    /// deserialization is not necessary, prefer to use `try_take_n_temp` instead.
    fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8], Self::PopError>;

    /// Attempt to take the next `ct` bytes from the serialized message.
    ///
    /// This variant does not guarantee that the returned value is borrowed from the input, so it
    /// cannot be used for zero-copy deserialization, but it also avoids needing to potentially
    /// allocate a data in a temporary buffer.
    ///
    /// This variant should be used instead of `try_take_n`
    /// if zero-copy deserialization is not necessary.
    ///
    /// It is only necessary to implement this method if the flavor requires storing data in a
    /// temporary buffer in order to implement the borrow semantics, e.g. the `std::io::Read`
    /// flavor.
    fn try_take_n_temp<'a>(&'a mut self, ct: usize) -> Result<&'a [u8], Self::PopError>
    where
        'de: 'a,
    {
        self.try_take_n(ct)
    }

    /// Complete the deserialization process.
    ///
    /// This should be called at the end of decoding.
    fn finalize(self) -> Result<Self::Remainder, Self::FinalizeError>;
}
