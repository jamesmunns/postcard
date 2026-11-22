use core::convert::Infallible;

use postcard::{
    BufferFull, SerializerError,
    ser_flavors::{Flavor, Patch, Slice},
    serialize_with_flavor,
};
use serde_core::Serialize;

#[cfg(feature = "alloc")]
extern crate alloc;

/// The maximum buffer size necessary to store a message of length `message_len`,
/// after cobs-encoding.
///
/// * +1 byte for every 254 data bytes (rounded up, and at least 1)
/// * +1 byte for `0x00` sentinel at the end
///
/// ```rust
/// use postcard_cobs::max_frame_len;
///
/// assert_eq!(max_frame_len(0), 2);
/// assert_eq!(max_frame_len(254), 256);
/// assert_eq!(max_frame_len(255), 258);
/// ```
#[inline]
pub const fn max_frame_len(message_len: usize) -> usize {
    let code_bytes = if message_len == 0 {
        1
    } else {
        message_len.div_ceil(254)
    };
    message_len.saturating_add(code_bytes).saturating_add(1)
}

/// The `Cobs` flavor implements [Consistent Overhead Byte Stuffing] on
/// the serialized data. The output of this flavor includes the termination/sentinel
/// byte of `0x00`.
///
/// This encoding is useful for framing data over a serial interface, such as a UART.
///
/// The inner flavor must implement [`Patch`].
///
/// ## Example
///
/// ```rust
/// use postcard::{ser_flavors::Slice, serialize_with_flavor};
/// use postcard_cobs::Cobs;
///
/// let mut buf = [0u8; 32];
/// let cobs = Cobs::try_new(Slice::new(&mut buf)).unwrap();
/// let used = serialize_with_flavor("Hi!", cobs).unwrap();
/// assert_eq!(used, &[
///     0x05,               // cobs len to next 0x00
///     0x03,               // "hi!".len()
///     b'H', b'i', b'!',   // "hi!"
///     0x00,               // end of frame sentinel
/// ]);
/// ```
///
/// [Consistent Overhead Byte Stuffing]: https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing
pub struct Cobs<B: Patch> {
    flav: B,
    /// Index in `flav.written_mut()` of the current block's code byte.
    /// Always inbounds, checked on pushes.
    code_idx: usize,
    /// The current block's code byte so far.
    code: u8,
}

impl<B: Patch> Cobs<B> {
    /// Create a new Cobs modifier Flavor. If there is insufficient space
    /// to push the leading header byte, the method will return an Error
    pub fn try_new(mut flav: B) -> Result<Self, B::PushError> {
        let code_idx = flav.written_mut().len();
        flav.try_push(0)?;
        Ok(Self {
            flav,
            code_idx,
            code: 1,
        })
    }

    /// End the current block and start the next one.
    fn next_block(&mut self) -> Result<(), B::PushError> {
        self.flav.written_mut()[self.code_idx] = self.code;
        self.flav.try_push(0)?;
        self.code_idx += usize::from(self.code);
        self.code = 1;
        Ok(())
    }
}

impl<B: Patch> Flavor for Cobs<B> {
    type Output = B::Output;
    type PushError = B::PushError;
    type FinalizeError = FinalizeError<B::PushError, B::FinalizeError>;

    #[inline]
    fn try_push(&mut self, data: u8) -> Result<(), B::PushError> {
        // If we're currently full, start the next block.
        if self.code == 0xFF {
            self.next_block()?;
        }
        // If this is a 0, we just need to start the next block.
        if data == 0 {
            return self.next_block();
        }
        // Failing that, push the data.
        self.flav.try_push(data)?;
        self.code += 1;
        Ok(())
    }

    fn finalize(mut self) -> Result<Self::Output, Self::FinalizeError> {
        self.flav.written_mut()[self.code_idx] = self.code;
        self.flav.try_push(0).map_err(FinalizeError::Push)?;
        self.flav.finalize().map_err(FinalizeError::Finalize)
    }
}

/// The error returned when finalizing a [`Cobs`] flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalizeError<PushErr, FinErr> {
    /// The inner flavor failed to push the terminating `0x00`.
    Push(PushErr),
    /// The inner flavor failed to finalize.
    Finalize(FinErr),
}

impl<PushErr, FinErr> core::fmt::Display for FinalizeError<PushErr, FinErr> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            FinalizeError::Push(_) => "Push",
            FinalizeError::Finalize(_) => "Finalize",
        })
    }
}

impl<PushErr, FinErr> core::error::Error for FinalizeError<PushErr, FinErr>
where
    PushErr: core::error::Error + 'static,
    FinErr: core::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            FinalizeError::Push(e) => Some(e),
            FinalizeError::Finalize(e) => Some(e),
        }
    }
}

/// Serialize a `T` to the given slice, with the resulting slice containing
/// data in a serialized then COBS encoded format. The terminating sentinel
/// `0x00` byte is included in the output buffer.
///
/// When successful, this function returns the slice containing the
/// serialized and encoded message.
///
/// ## Example
///
/// ```rust
/// use postcard_cobs::to_slice_cobs;
/// let mut buf = [0u8; 32];
///
/// let used = to_slice_cobs(&false, &mut buf).unwrap();
/// assert_eq!(used, &[0x01, 0x01, 0x00]);
///
/// let used = to_slice_cobs("1", &mut buf).unwrap();
/// assert_eq!(used, &[0x03, 0x01, b'1', 0x00]);
///
/// let used = to_slice_cobs("Hi!", &mut buf).unwrap();
/// assert_eq!(used, &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
///
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let used = to_slice_cobs(data, &mut buf).unwrap();
/// assert_eq!(used, &[0x03, 0x04, 0x01, 0x03, 0x20, 0x30, 0x00]);
/// ```
pub fn to_slice_cobs<'a, T>(
    value: &T,
    buf: &'a mut [u8],
) -> Result<&'a mut [u8], SerializerError<BufferFull, FinalizeError<BufferFull, Infallible>>>
where
    T: Serialize + ?Sized,
{
    let cobs = Cobs::try_new(Slice::new(buf)).map_err(SerializerError::PushError)?;
    serialize_with_flavor(value, cobs)
}

/// Serialize and COBS encode a `T` to an `alloc::vec::Vec<u8>`.
///
/// The terminating sentinel `0x00` byte is included in the output.
///
/// ## Example
///
/// ```rust
/// use postcard_cobs::to_vec_cobs;
///
/// let ser: Vec<u8> = to_vec_cobs(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x02, 0x01, 0x00]);
///
/// let ser: Vec<u8> = to_vec_cobs("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn to_vec_cobs<T>(
    value: &T,
) -> Result<alloc::vec::Vec<u8>, SerializerError<Infallible, FinalizeError<Infallible, Infallible>>>
where
    T: Serialize + ?Sized,
{
    let cobs = Cobs::try_new(alloc::vec::Vec::new()).map_err(SerializerError::PushError)?;
    serialize_with_flavor(value, cobs)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn too_small() {
        // No room for the leading code byte.
        let mut buf = [0u8; 0];
        assert!(matches!(
            to_slice_cobs(&0u8, &mut buf),
            Err(SerializerError::PushError(BufferFull))
        ));

        // Room for the encoded byte, but not the terminator.
        let mut buf = [0u8; 2];
        assert!(matches!(
            to_slice_cobs(&1u8, &mut buf),
            Err(SerializerError::FinalizeError(FinalizeError::Push(
                BufferFull
            )))
        ));
    }

    /// Ignoring an error never panics.
    #[test]
    fn ignored_errors() {
        for (block_len, byte) in [(0, 0), (254, 1), (254, 0)] {
            let mut buf = [0u8; 255];
            let mut cobs = Cobs::try_new(Slice::new(&mut buf[..block_len + 1])).unwrap();
            for _ in 0..block_len {
                cobs.try_push(1).unwrap();
            }
            for _ in 0..3 {
                assert_eq!(cobs.try_push(byte), Err(BufferFull));
            }
            assert_eq!(cobs.finalize(), Err(FinalizeError::Push(BufferFull)));
        }
    }

    /// Same check as [`ignored_errors`], from a [`Serialize`] impl.
    #[test]
    fn ignored_errors_serialize() {
        use serde::ser::SerializeTuple;

        struct Zeros;

        impl Serialize for Zeros {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let mut tuple = s.serialize_tuple(4)?;
                for _ in 0..4 {
                    let _ = tuple.serialize_element(&0u8);
                }
                tuple.end()
            }
        }

        for len in 0..6 {
            assert!(to_slice_cobs(&Zeros, &mut [0u8; 6][..len]).is_err());
        }
        assert!(to_slice_cobs(&Zeros, &mut [0u8; 6]).is_ok());
    }

    #[test]
    fn max_frame_len_saturates() {
        assert_eq!(max_frame_len(usize::MAX), usize::MAX);
        assert_eq!(max_frame_len(usize::MAX - 1), usize::MAX);
    }

    /// Data already in the inner flavor is left unchanged.
    #[cfg(feature = "alloc")]
    #[test]
    fn existing_data() {
        let cobs = Cobs::try_new(vec![0xFFu8, 0xFF]).unwrap();
        let vec = serialize_with_flavor("Hi!", cobs).unwrap();
        assert_eq!(vec, &[0xFF, 0xFF, 0x05, 0x03, b'H', b'i', b'!', 0x00]);
    }
}
