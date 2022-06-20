//! # Deserialization Flavors
//!
//! "Flavors" in `postcard` are used as modifiers to the serialization or deserialization
//! process. Flavors typically modify one or both of the following:
//!
//! 1. The source medium of the deserialization, e.g. whether the data is serialized from a `[u8]` slice, or some other container
//! 2. The format of the deserialization, such as if the original data is encoded in a COBS format, contains a CRC32 checksum
//!      appended to the message, etc.
//!
//! Flavors are implemented using the [`Flavor`] trait, which acts as a "middleware" for retrieving the bytes before they
//! are passed to `serde` for deserialization
//!
//! Multiple flavors may be combined to obtain a desired combination of behavior and storage.
//! When flavors are combined, it is expected that the storage flavor (such as [`Slice`]) is the innermost flavor.
//!
//! Custom flavors may be defined by users of the `postcard` crate, however some commonly useful flavors have been provided in
//! this module. If you think your custom flavor would be useful to others, PRs adding flavors are very welcome!
//!
//! ## Usability
//!
//! Flavors may not always be convenient to use directly, as they may expose some implementation details of how the
//! inner workings of the flavor behaves. It is typical to provide a convenience method for using a flavor, to prevent
//! the user from having to specify generic parameters, setting correct initialization values, or handling the output of
//! the flavor correctly. See `postcard::from_bytes()` for an example of this.
//!
//! ## When to use (multiple) flavors
//!
//! Combining flavors are nice for convenience, as they perform potentially multiple steps of
//! serialization at one time.
//!
//! This can often be more memory efficient, as intermediate buffers are not typically required.
//!
//! ## When NOT to use (multiple) flavors
//!
//! The downside of passing deserialization through multiple steps is that it is typically slower than
//! performing each step serially. Said simply, "cobs decoding while deserializing" is often slower
//! than "cobs decode then deserialize", due to the ability to handle longer "runs" of data in each
//! stage. The downside is that if these stages can not be performed in-place on the buffer, you
//! will need additional buffers for each stage.
//!
//! Additionally, deserializating flavors can be more restrictive or difficult to work with than
//! serialization flavors, as deserialization may require that the deserialized types borrow some
//! portion of the original message.
//!
//! ## Examples
//!
//! ### Using a single flavor
//!
//! In the first example, we use the `Slice` flavor, to retrieve the serialized output from a `[u8]` slice.
//! No other modification is made to the serialization process.
//!
//! ```rust
//! use postcard::{
//!     de_flavors::Slice,
//!     Deserializer,
//! };
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Debug, PartialEq)]
//! struct Tup(u8, u8, u8);
//!
//! let msg = [0x04, 0x00, 0x04, 0x01, 0x02, 0x03];
//! let slice = Slice::new(&msg);
//! let mut deserializer = Deserializer::from_flavor(slice);
//! let t = Tup::deserialize(&mut deserializer).unwrap();
//! assert_eq!(t, Tup(4, 0, 4));
//! let remainder = deserializer.finalize().unwrap();
//! assert_eq!(remainder, &[1, 2, 3]);
//! ```

use crate::{Error, Result};
use core::marker::PhantomData;

/// The deserialization Flavor trait
///
/// This is used as the primary way to decode serialized data from some kind of buffer,
/// or modify that data in a middleware style pattern.
///
/// See the module level docs for an example of how flavors are used.
pub trait Flavor<'de>: 'de {
    /// The remaining data of this flavor after deserializing has completed.
    ///
    /// Typically, this includes the remaining buffer that was not used for
    /// deserialization, and in cases of more complex flavors, any additional
    /// information that was decoded or otherwise calculated during
    /// the deserialization process.
    type Remainder: 'de;

    /// The source of data retrieved for deserialization.
    ///
    /// This is typically some sort of data buffer, or another Flavor, when
    /// chained behavior is desired
    type Source: 'de;

    /// Obtain the next byte for deserialization
    fn pop(&mut self) -> Result<u8>;

    /// Attempt to take the next `ct` bytes from the serialized message
    fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8]>;

    /// Complete the deserialization process.
    ///
    /// This is typically called separately, after the `serde` deserialization
    /// has completed.
    fn finalize(self) -> Result<Self::Remainder>;
}

/// A simple [`Flavor`] representing the deserialization from a borrowed slice
pub struct Slice<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    pub(crate) cursor: *const u8,
    pub(crate) end: *const u8,
    pub(crate) _pl: PhantomData<&'de [u8]>,
}

impl<'de> Slice<'de> {
    /// Create a new [Slice] from the given buffer
    pub fn new(sli: &'de [u8]) -> Self {
        Self {
            cursor: sli.as_ptr(),
            end: unsafe { sli.as_ptr().add(sli.len()) },
            _pl: PhantomData,
        }
    }
}

impl<'de> Flavor<'de> for Slice<'de> {
    type Remainder = &'de [u8];
    type Source = &'de [u8];

    #[inline]
    fn pop(&mut self) -> Result<u8> {
        if self.cursor == self.end {
            Err(Error::DeserializeUnexpectedEnd)
        } else {
            unsafe {
                let res = Ok(*self.cursor);
                self.cursor = self.cursor.add(1);
                res
            }
        }
    }

    #[inline]
    fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8]> {
        let remain = (self.end as usize) - (self.cursor as usize);
        if remain < ct {
            Err(Error::DeserializeUnexpectedEnd)
        } else {
            unsafe {
                let sli = core::slice::from_raw_parts(self.cursor, ct);
                self.cursor = self.cursor.add(ct);
                Ok(sli)
            }
        }
    }

    /// Return the remaining (unused) bytes in the Deserializer
    fn finalize(self) -> Result<&'de [u8]> {
        let remain = (self.end as usize) - (self.cursor as usize);
        unsafe { Ok(core::slice::from_raw_parts(self.cursor, remain)) }
    }
}

// This is a terrible checksum implementation to make sure that we can effectively
// use the deserialization flavor. This is kept as a test (and not published)
// because an 8-bit checksum is not ACTUALLY useful for almost anything.
//
// You could certainly do something similar with a CRC32, cryptographic sig,
// or something else
#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    struct Checksum<'de, F>
    where
        F: Flavor<'de> + 'de,
    {
        flav: F,
        checksum: u8,
        _plt: PhantomData<&'de ()>,
    }

    impl<'de, F> Checksum<'de, F>
    where
        F: Flavor<'de> + 'de,
    {
        pub fn from_flav(flav: F) -> Self {
            Self {
                flav,
                checksum: 0,
                _plt: PhantomData,
            }
        }
    }

    impl<'de, F> Flavor<'de> for Checksum<'de, F>
    where
        F: Flavor<'de> + 'de,
    {
        type Remainder = (<F as Flavor<'de>>::Remainder, u8);
        type Source = F;

        fn pop(&mut self) -> Result<u8> {
            match self.flav.pop() {
                Ok(u) => {
                    self.checksum = self.checksum.wrapping_add(u);
                    Ok(u)
                }
                Err(e) => Err(e),
            }
        }
        fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8]> {
            match self.flav.try_take_n(ct) {
                Ok(u) => {
                    u.iter().for_each(|u| {
                        self.checksum = self.checksum.wrapping_add(*u);
                    });
                    Ok(u)
                }
                Err(e) => Err(e),
            }
        }
        fn finalize(self) -> Result<Self::Remainder> {
            Ok((self.flav.finalize()?, self.checksum))
        }
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    pub struct SomeData<'a> {
        #[serde(borrow)]
        sli: &'a [u8],
        sts: &'a str,
        foo: u64,
        bar: u128,
    }

    #[test]
    fn smoke() {
        const EXPECTED: &[u8] = &[
            4, 255, 1, 34, 51, 19, 116, 104, 105, 115, 32, 105, 115, 32, 97, 32, 103, 111, 111,
            100, 32, 116, 101, 115, 116, 170, 213, 170, 213, 170, 213, 170, 213, 170, 1, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127,
        ];

        // Calculate simple 8-bit checksum
        let mut check: u8 = 0;
        EXPECTED.iter().for_each(|u| check = check.wrapping_add(*u));

        let mut buf = [0u8; 256];
        let data = SomeData {
            sli: &[0xFF, 0x01, 0x22, 0x33],
            sts: "this is a good test",
            foo: (u64::MAX / 3) * 2,
            bar: u128::MAX / 4,
        };
        let used = crate::to_slice(&data, &mut buf).unwrap();
        assert_eq!(used, EXPECTED);
        let used = used.len();

        // Put the checksum at the end
        buf[used] = check;

        let mut deser = crate::de::Deserializer::from_flavor(Checksum::from_flav(Slice::new(&buf)));

        let t = SomeData::<'_>::deserialize(&mut deser).unwrap();
        assert_eq!(t, data);

        // Normally, you'd probably expect the check
        let (rem, cksm) = deser.finalize().unwrap();

        // The pre-calculated checksum we stuffed at the end is the
        // first "unused" byte
        assert_eq!(rem[0], check);

        // the one we calculated during serialization matches the
        // pre-calculated one
        assert_eq!(cksm, check);
    }
}
