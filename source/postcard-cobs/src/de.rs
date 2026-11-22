use core::convert::Infallible;

use postcard::{DeserializerError, UnexpectedEnd, from_bytes};
use serde_core::Deserialize;

/// The error returned when deserializing from a COBS frame.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeserializeError {
    /// The input was empty or only contained `0x00` bytes.
    NoFrame,
    /// The frame ended in the middle of a block.
    InvalidFrame,
    /// The input ended before the frame's terminating `0x00`.
    Truncated,
    /// The frame was decoded, but its message could not be deserialized.
    Deserialize(DeserializerError<UnexpectedEnd, Infallible>),
}

impl core::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            DeserializeError::NoFrame => "NoFrame",
            DeserializeError::InvalidFrame => "InvalidFrame",
            DeserializeError::Truncated => "Truncated",
            DeserializeError::Deserialize(_) => "Deserialize",
        })
    }
}

impl core::error::Error for DeserializeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            DeserializeError::Deserialize(e) => Some(e),
            _ => None,
        }
    }
}

/// The error and remainder data returned by [`take_from_bytes_cobs`].
#[derive(Debug, PartialEq, Eq)]
pub struct TakeFromBytesCobsError<'a> {
    /// What went wrong.
    pub error: DeserializeError,
    /// The input after the bad frame, to continue with.
    ///
    /// For [`DeserializeError::NoFrame`], this is empty.
    /// For [`DeserializeError::Truncated`] it is the whole input so that it can be
    /// retried once more bytes have been received.
    pub remaining: &'a mut [u8],
}

impl core::fmt::Display for TakeFromBytesCobsError<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(f)
    }
}

impl core::error::Error for TakeFromBytesCobsError<'_> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.error.source()
    }
}

/// Deserialize a message of type `T` from a cobs-encoded byte slice.
///
/// The unused portion (if any) of the byte slice is not returned. The used portion may be
/// modified while decoding, even if an error is returned.
///
/// Extra leading `0x00` bytes are skipped, and the frame's terminating `0x00` is required.
///
/// ## Example
///
/// ```rust
/// use postcard_cobs::from_bytes_cobs;
///
/// let mut frame = [0x05, 0x03, b'H', b'i', b'!', 0x00];
/// let text: &str = from_bytes_cobs(&mut frame).unwrap();
/// assert_eq!(text, "Hi!");
/// ```
pub fn from_bytes_cobs<'a, T>(s: &'a mut [u8]) -> Result<T, DeserializeError>
where
    T: Deserialize<'a>,
{
    match take_from_bytes_cobs(s) {
        Ok((t, _)) => Ok(t),
        Err(e) => Err(e.error),
    }
}

/// Deserialize a message of type `T` from a cobs-encoded byte slice.
///
/// The unused portion (if any) of the byte slice is returned for further usage. The used
/// portion may be modified while decoding, even if an error is returned.
///
/// Extra leading `0x00` bytes are skipped, and the frame's terminating `0x00` is required.
/// The unused portion is also returned on error, see [`TakeFromBytesCobsError::remaining`].
///
/// If you are processing a stream of data, consider using
/// [`CobsAccumulator`][crate::CobsAccumulator] instead.
///
/// ## Example
///
/// ```rust
/// use postcard_cobs::{DeserializeError, TakeFromBytesCobsError, take_from_bytes_cobs};
///
/// // A frame, a malformed frame, another frame, and the start of one more
/// let mut stream = [
///     0x04, 0x02, b'H', b'i', 0x00,
///     0x09, 0x01, 0x00, 0x04, 0x02, b'y', b'o', 0x00, 0x04, 0x02,
/// ];
///
/// let (text, remaining) = take_from_bytes_cobs::<&str>(&mut stream).unwrap();
///
/// assert_eq!(text, "Hi");
/// assert_eq!(remaining, &[0x09, 0x01, 0x00, 0x04, 0x02, b'y', b'o', 0x00, 0x04, 0x02]);
/// ```
pub fn take_from_bytes_cobs<'a, T>(
    s: &'a mut [u8],
) -> Result<(T, &'a mut [u8]), TakeFromBytesCobsError<'a>>
where
    T: Deserialize<'a>,
{
    // Skip any leading 0x00s
    let Some(start) = s.iter().position(|b| *b != 0) else {
        return Err(TakeFromBytesCobsError {
            error: DeserializeError::NoFrame,
            remaining: &mut [],
        });
    };
    // Find the terminal 0x00
    let Some(len) = s[start..].iter().position(|b| *b == 0) else {
        return Err(TakeFromBytesCobsError {
            error: DeserializeError::Truncated,
            remaining: s,
        });
    };
    // Split to the end, including bytes before `start` (if any). We DO take the
    // sentinel byte here (so it's not in the remainder), but don't pass it into
    // `from_frame`.
    let (used, remaining) = s.split_at_mut(start + len + 1);
    match from_frame(&mut used[start..start + len]) {
        Ok(t) => Ok((t, remaining)),
        Err(error) => Err(TakeFromBytesCobsError { error, remaining }),
    }
}

/// Decode a frame in place, then deserialize it. `frame` should contain no zeros.
pub(crate) fn from_frame<'a, T>(frame: &'a mut [u8]) -> Result<T, DeserializeError>
where
    T: Deserialize<'a>,
{
    let message = decode_frame(frame)?;
    // Decoded, now deserialize
    from_bytes(message).map_err(DeserializeError::Deserialize)
}

/// Decode a frame in place, returning the message.
///
/// `frame` should NOT contain any zeros.
pub(crate) fn decode_frame(frame: &mut [u8]) -> Result<&mut [u8], DeserializeError> {
    // Each block of `code` bytes decodes to at most `code` bytes, so `dst` never passes `src`.
    let (mut src, mut dst) = (0, 0);
    while src < frame.len() {
        // Read the code block..
        let code = usize::from(frame[src]);
        // ..validate it...
        if code == 0 || src + code > frame.len() {
            return Err(DeserializeError::InvalidFrame);
        }
        // slide data to the left
        frame.copy_within(src + 1..src + code, dst);
        // increment src/dst indexes
        dst += code - 1;
        src += code;
        // An 0xFF is an "extra code block" case
        // src == frame.len(): end of frame
        // otherwise: a data zero
        if code != 0xFF && src < frame.len() {
            frame[dst] = 0;
            dst += 1;
        }
    }
    Ok(&mut frame[..dst])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_error() {
        let mut buf = [0x02, 0x80, 0x00, 0x02, 0x2A, 0x00];
        let err = take_from_bytes_cobs::<u16>(&mut buf).unwrap_err();
        assert!(matches!(err.error, DeserializeError::Deserialize(_)));
        assert_eq!(err.remaining, &[0x02, 0x2A, 0x00]);
    }

    #[test]
    fn zero_code() {
        for frame in [&mut [0][..], &mut [0, 1], &mut [2, 1, 0]] {
            let err = from_frame::<()>(frame).unwrap_err();
            assert_eq!(err, DeserializeError::InvalidFrame);
        }
    }

    #[test]
    fn error_sources() {
        use core::error::Error;
        let err = from_bytes_cobs::<u16>(&mut [0x02, 0x80, 0x00]).unwrap_err();
        assert_eq!(err.to_string(), "Deserialize");
        assert_eq!(err.source().unwrap().to_string(), "Pop");

        let err = from_bytes_cobs::<u8>(&mut [0x09, 0x00]).unwrap_err();
        assert_eq!(err.to_string(), "InvalidFrame");
        assert!(err.source().is_none());
    }
}
