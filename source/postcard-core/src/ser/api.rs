//! Flavor traits and error types for serialization

/// The serialization buffer is full
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt-v1", derive(defmt::Format))]
pub struct BufferFull;

impl core::fmt::Display for BufferFull {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BufferFull")
    }
}

impl core::error::Error for BufferFull {}

/// The serialization Flavor trait
///
/// This is used as the primary way to encode serialized data into some kind of buffer,
/// or modify that data in a middleware style pattern.
///
/// See the module level docs for an example of how flavors are used.
pub trait Flavor {
    /// The `Output` type is what this storage "resolves" to when the serialization is complete,
    /// such as a slice or a Vec of some sort.
    type Output;

    /// The error type specific to pushing methods.
    ///
    /// This includes [`Self::try_extend`] and [`Self::try_push`].
    ///
    /// If this type cannot error when pushing, e.g. with a `Vec`, consider using
    /// [`Infallible`](core::convert::Infallible). If this type can only fail due
    /// to exhausting available space, consider using [`BufferFull`].
    type PushError: core::error::Error + 'static;

    /// The error type specific to [`Self::finalize`].
    ///
    /// If this type cannot error when pushing, e.g. for storage flavors that don't
    /// perform any meaningful finalization actions, consider using
    /// [`Infallible`](core::convert::Infallible).
    type FinalizeError: core::error::Error + 'static;

    /// Override this method when you want to customize processing
    /// multiple bytes at once, such as copying a slice to the output,
    /// rather than iterating over one byte at a time.
    #[inline]
    fn try_extend(&mut self, data: &[u8]) -> Result<(), Self::PushError> {
        data.iter().try_for_each(|d| self.try_push(*d))
    }

    /// Push a single byte to be modified and/or stored.
    fn try_push(&mut self, data: u8) -> Result<(), Self::PushError>;

    /// Finalize the serialization process.
    fn finalize(self) -> Result<Self::Output, Self::FinalizeError>;
}

/// Implement Flavor for non-owned flavors.
///
/// [`Flavor::finalize`] does nothing and returns `()`. Users should call
/// finalize on the borrowed flavor once all items have been pushed.
impl<F: Flavor + ?Sized> Flavor for &mut F {
    type Output = ();
    type PushError = F::PushError;
    type FinalizeError = core::convert::Infallible;

    #[inline]
    fn try_extend(&mut self, data: &[u8]) -> Result<(), Self::PushError> {
        (**self).try_extend(data)
    }

    #[inline]
    fn try_push(&mut self, data: u8) -> Result<(), Self::PushError> {
        (**self).try_push(data)
    }

    #[inline]
    fn finalize(self) -> Result<(), Self::FinalizeError> {
        Ok(())
    }
}

/// A storage [`Flavor`] whose already pushed bytes can be modified in place
///
/// This is intended for modifier flavors that need to go back and fill in a byte
/// they could not know when it was pushed, such as the code bytes of a COBS
/// encoder. Modifier flavors that transform or observe the bytes passing through
/// them, such as a CRC, should not implement this, as modifying earlier bytes
/// would potentially invalidate the contents.
///
/// A modifier flavor that needs `Patch` should always sit directly on top of the
/// storage flavor.
///
/// Positions are relative to the slice returned by [`Patch::written_mut`].
/// A modifier flavor that may be given a storage flavor that already contains
/// data should record `written_mut().len()` when it starts, and only modify
/// bytes after that point.
pub trait Patch: Flavor {
    /// All bytes pushed to this flavor so far, in the order they were pushed.
    ///
    /// Implementors must not include storage that was never pushed, such as the
    /// unused remainder of a buffer. This may include bytes that were pushed
    /// before the current modifier flavor started.
    fn written_mut(&mut self) -> &mut [u8];
}

impl<P: Patch + ?Sized> Patch for &mut P {
    #[inline]
    fn written_mut(&mut self) -> &mut [u8] {
        (**self).written_mut()
    }
}
