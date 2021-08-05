//! An accumulator used to collect chunked COBS data and deserialize it.

use core::mem::MaybeUninit;
use serde::Deserialize;

/// An accumulator used to collect chunked COBS data and deserialize it.
pub struct CobsAccumulator<const N: usize> {
    buf: [u8; N],
    idx: usize,
}

/// The result of feeding the accumulator.
pub enum FeedResult<'a, T> {
    /// Consumed all data, still pending.
    Consumed,

    /// Buffer was filled. Contains remaining section of input, if any.
    OverFull(&'a [u8]),

    /// Reached end of chunk, but deserialization failed. Contains remaining section of input, if.
    /// any
    DeserError(&'a [u8]),

    /// Deserialization complete. Contains deserialized data and remaining section of input, if any.
    Success {
        /// Deserialize data.
        data: T,

        /// Remaining data left in the buffer after deserializing.
        remaining: &'a [u8],
    },
}

impl<const N: usize> CobsAccumulator<N> {
    /// Create a new accumulator.
    pub fn new() -> Self {
        CobsAccumulator {
            buf: [0; N],
            idx: 0,
        }
    }

    /// Appends data to the internal buffer and attempts to deserialize the accumulated data into
    /// `T`.
    pub fn feed<'a, T>(&mut self, input: &'a [u8]) -> FeedResult<'a, T>
    where
        T: for<'de> Deserialize<'de>,
    {
        if input.is_empty() {
            return FeedResult::Consumed;
        }

        let zero_pos = input.iter().position(|&i| i == 0);

        if let Some(n) = zero_pos {
            // Yes! We have an end of message here.
            // Add one to include the zero in the "take" portion
            // of the buffer, rather than in "release".
            let (take, release) = input.split_at(n + 1);

            // Does it fit?
            if (self.idx + n) <= N {
                // Aw yiss - add to array
                self.extend_unchecked(take);

                let retval = match crate::from_bytes_cobs::<T>(&mut self.buf[..self.idx]) {
                    Ok(t) => FeedResult::Success {
                        data: t,
                        remaining: release,
                    },
                    Err(_) => FeedResult::DeserError(release),
                };
                self.idx = 0;
                retval
            } else {
                self.idx = 0;
                FeedResult::OverFull(release)
            }
        } else {
            // Does it fit?
            if (self.idx + input.len()) > N {
                // nope
                let new_start = N - self.idx;
                self.idx = 0;
                FeedResult::OverFull(&input[new_start..])
            } else {
                // yup!
                self.extend_unchecked(input);
                FeedResult::Consumed
            }
        }
    }

    /// Extend the internal buffer with the given input.
    ///
    /// # Panics
    ///
    /// Will panic if the input does not fit in the internal buffer.
    fn extend_unchecked(&mut self, input: &[u8]) {
        let new_end = self.idx + input.len();
        self.buf[self.idx..new_end].copy_from_slice(input);
        self.idx = new_end;
    }
}

#[test]
fn loop_test() {
    #[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Demo {
        a: u32,
        b: u8,
    }

    let mut raw_buf = [0u8; 64];
    let mut cobs_buf: CobsAccumulator<64> = CobsAccumulator::new();

    let ser = crate::to_slice_cobs(&Demo { a: 10, b: 20 }, &mut raw_buf).unwrap();

    if let FeedResult::Success { data, remaining } = cobs_buf.feed(ser) {
        assert_eq!(Demo { a: 10, b: 20 }, data);
        assert_eq!(remaining.len(), 0);
    } else {
        panic!()
    }
}
