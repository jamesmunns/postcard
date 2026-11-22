use postcard::from_bytes;
use serde_core::Deserialize;

use crate::de::{DeserializeError, decode_frame};

/// An accumulator used to collect chunked COBS data and deserialize it.
///
/// This is often useful when you receive "parts" of the message at a time, for example when draining
/// a serial port buffer that may not contain an entire uninterrupted message.
///
/// `N` should be large enough to hold the maximum encoded message length, including the terminal
/// sentinel `0x00`.
///
/// See [`max_frame_len`](crate::max_frame_len) for calculating "how big is big enough" for `N`.
///
/// # Examples
///
/// Deserialize a struct by reading chunks from a [`Read`]er.
///
/// ```rust
/// use postcard_cobs::{CobsAccumulator, FeedResult};
/// use serde::Deserialize;
/// use std::io::Read;
///
/// # let mut input_buf = [0u8; 256];
/// # #[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq)]
/// # struct MyData {
/// #     a: u32,
/// #     b: bool,
/// #     c: [u8; 16],
/// # }
/// let input = /* Anything that implements the `Read` trait */
/// # postcard_cobs::to_slice_cobs(&MyData {
/// #     a: 0xabcdef00,
/// #     b: true,
/// #     c: [0xab; 16],
/// # }, &mut input_buf).unwrap();
/// # let mut input = &input[..];
/// # let mut received = 0;
///
/// let mut raw_buf = [0u8; 32];
/// let mut cobs_buf: CobsAccumulator<256> = CobsAccumulator::new();
///
/// while let Ok(ct) = input.read(&mut raw_buf) {
///     // Finished reading input
///     if ct == 0 {
///         break;
///     }
///
///     let buf = &raw_buf[..ct];
///     let mut window = &buf[..];
///
///     'cobs: while !window.is_empty() {
///         window = match cobs_buf.feed::<MyData>(&window) {
///             FeedResult::Consumed => break 'cobs,
///             FeedResult::OverFull(new_wind) => new_wind,
///             FeedResult::DeserError { remaining, .. } => remaining,
///             FeedResult::Success { data, remaining } => {
///                 // Do something with `data: MyData` here.
///
///                 dbg!(data);
/// #               received += 1;
///
///                 remaining
///             }
///         };
///     }
/// }
/// # assert_eq!(received, 1);
/// ```
///
/// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
#[derive(Debug)]
pub struct CobsAccumulator<const N: usize> {
    buf: [u8; N],
    /// The length of the frame so far (could be >N, if we're `OverFull`)
    idx: usize,
}

/// The result of feeding the accumulator.
#[derive(Debug, PartialEq, Eq)]
pub enum FeedResult<'a, T> {
    /// Consumed all data, still pending.
    Consumed,

    /// Reached the end of a frame that didn't fit in the buffer.
    /// Contains remaining section of input, if any.
    ///
    /// A frame that doesn't fit is only reported once its terminating `0x00` is received.
    OverFull(&'a [u8]),

    /// Reached the end of a frame, but it could not be decoded, or its message could not be
    /// deserialized.
    DeserError {
        /// Why the frame could not be decoded/deserialized.
        error: DeserializeError,

        /// Remaining section of input, if any.
        remaining: &'a [u8],
    },

    /// Deserialization complete.
    /// Contains deserialized data and remaining section of input, if any.
    Success {
        /// Deserialized data.
        data: T,

        /// Remaining section of input, if any.
        remaining: &'a [u8],
    },
}

impl<const N: usize> Default for CobsAccumulator<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> CobsAccumulator<N> {
    /// Create a new accumulator.
    pub const fn new() -> Self {
        CobsAccumulator {
            buf: [0; N],
            idx: 0,
        }
    }

    /// Appends data to the internal buffer and attempts to deserialize the accumulated data into
    /// `T`.
    #[inline]
    pub fn feed<'a, T>(&mut self, input: &'a [u8]) -> FeedResult<'a, T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.feed_ref(input)
    }

    /// Appends data to the internal buffer and attempts to deserialize the accumulated data into
    /// `T`.
    ///
    /// This differs from feed, as it allows the `T` to reference data within the internal buffer, but
    /// mutably borrows the accumulator for the lifetime of the deserialization.
    /// If `T` does not require the reference, the borrow of `self` ends at the end of the function.
    pub fn feed_ref<'de, 'a, T>(&'de mut self, input: &'a [u8]) -> FeedResult<'a, T>
    where
        T: Deserialize<'de>,
    {
        match self.feed_raw(input) {
            FeedResult::Consumed => FeedResult::Consumed,
            FeedResult::OverFull(remaining) => FeedResult::OverFull(remaining),
            FeedResult::DeserError { error, remaining } => {
                FeedResult::DeserError { error, remaining }
            }
            FeedResult::Success { data, remaining } => match from_bytes(data) {
                Ok(data) => FeedResult::Success { data, remaining },
                Err(e) => FeedResult::DeserError {
                    error: DeserializeError::Deserialize(e),
                    remaining,
                },
            },
        }
    }

    /// Appends data to the internal buffer and attempts to decode the accumulated data, without
    /// deserializing it.
    ///
    /// This method can be used when you don't want to immediately deserialize the contents, e.g.
    /// if you need to extract or check a header/footer first.
    ///
    /// As with [`Self::feed_ref`], the message references data within the internal buffer.
    ///
    /// ```rust
    /// use postcard_cobs::{CobsAccumulator, FeedResult};
    ///
    /// let mut acc: CobsAccumulator<8> = CobsAccumulator::new();
    /// assert_eq!(
    ///     acc.feed_raw(&[0x02, 0x11, 0x02, 0x22, 0x00, 0x33]),
    ///     FeedResult::Success {
    ///         data: &[0x11, 0x00, 0x22][..],
    ///         remaining: &[0x33],
    ///     },
    /// );
    /// ```
    pub fn feed_raw<'de, 'a>(&'de mut self, input: &'a [u8]) -> FeedResult<'a, &'de [u8]> {
        let mut input = input;
        while let Some(len) = input.iter().position(|b| *b == 0) {
            let remaining = &input[len + 1..];
            self.extend(&input[..len]);
            let len = core::mem::take(&mut self.idx);
            if len == 0 {
                input = remaining;
                continue;
            }
            let Some(frame) = self.buf.get_mut(..len) else {
                return FeedResult::OverFull(remaining);
            };
            return match decode_frame(frame) {
                Ok(data) => FeedResult::Success { data, remaining },
                Err(error) => FeedResult::DeserError { error, remaining },
            };
        }
        self.extend(input);
        FeedResult::Consumed
    }

    /// Append to the frame.
    ///
    /// If the bytes don't fit, we don't store the bytes, and only increment idx.
    fn extend(&mut self, input: &[u8]) {
        let end = self.idx.saturating_add(input.len());
        if let Some(dest) = self.buf.get_mut(self.idx..end) {
            dest.copy_from_slice(input);
        }
        self.idx = end;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    /// COBS encode `a` then `b` back to back into `buf`, returning the used part.
    fn two_frames<'a, A: serde::Serialize, B: serde::Serialize>(
        a: &A,
        b: &B,
        buf: &'a mut [u8],
    ) -> &'a [u8] {
        let first = crate::to_slice_cobs(a, buf).unwrap().len();
        let second = crate::to_slice_cobs(b, &mut buf[first..]).unwrap().len();
        &buf[..first + second]
    }

    #[test]
    fn double_loop_test_ref() {
        #[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Demo<'a> {
            a: u32,
            b: u8,
            c: &'a str,
        }

        let mut cobs_buf: CobsAccumulator<64> = CobsAccumulator::new();

        let mut buf = [0u8; 128];
        let ser = two_frames(
            &Demo {
                a: 10,
                b: 20,
                c: "test",
            },
            &Demo {
                a: 256854231,
                b: 115,
                c: "different test",
            },
            &mut buf,
        );

        let (data, ser) = if let FeedResult::Success { data, remaining } = cobs_buf.feed_ref(ser) {
            (data, remaining)
        } else {
            panic!()
        };

        assert!(
            Demo {
                a: 10,
                b: 20,
                c: "test"
            } == data
        );

        let demo2 = if let FeedResult::Success { data, remaining } = cobs_buf.feed_ref(ser) {
            assert!(remaining.is_empty());
            data
        } else {
            panic!()
        };

        // Uncommenting the below line causes the test to no-longer compile, as cobs_buf would then be mutably borrowed twice
        //assert!(Demo { a: 10, b: 20, c : "test" } == data);

        assert!(
            Demo {
                a: 256854231,
                b: 115,
                c: "different test"
            } == demo2
        );
    }

    #[test]
    fn raw() {
        let mut acc: CobsAccumulator<8> = CobsAccumulator::new();
        assert_eq!(acc.feed_raw(&[0, 3, 1]), FeedResult::Consumed);
        assert_eq!(
            acc.feed_raw(&[2, 2, 3, 0, 9]),
            FeedResult::Success {
                data: &[1, 2, 0, 3][..],
                remaining: &[9]
            }
        );
        // The '9' above was not consumed.
        assert_eq!(
            acc.feed_raw(&[3, 1, 0, 9]),
            FeedResult::DeserError {
                error: DeserializeError::InvalidFrame,
                remaining: &[9]
            }
        );
        assert_eq!(
            acc.feed_raw(&[1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 9]),
            FeedResult::OverFull(&[9])
        );
        // An empty message
        assert_eq!(
            acc.feed_raw(&[1, 0]),
            FeedResult::Success {
                data: &[][..],
                remaining: &[]
            }
        );
    }

    #[test]
    fn deserialize_error() {
        let mut acc: CobsAccumulator<4> = CobsAccumulator::new();
        assert!(matches!(
            acc.feed::<u8>(&[1, 0, 7]),
            FeedResult::DeserError {
                error: DeserializeError::Deserialize(_),
                remaining: &[7]
            }
        ));
    }
}
