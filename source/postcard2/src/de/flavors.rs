//! # Deserialization Flavors
//!
//! "Flavors" in `postcard` are used as modifiers to the serialization or deserialization
//! process. Flavors typically modify one or both of the following:
//!
//! 1. The source medium of the deserialization, e.g. whether the data is serialized from a `[u8]` slice, or some other container
//! 2. The format of the deserialization, such as if the original data is encoded in a COBS format, contains a CRC32 checksum
//!    appended to the message, etc.
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
//! the flavor correctly. See `postcard2::from_bytes()` for an example of this.
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
//! use postcard2::{
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

use core::{convert::Infallible, marker::PhantomData};
pub use postcard_core::de::{Flavor, UnexpectedEnd};

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
        let range = sli.as_ptr_range();
        Self {
            cursor: range.start,
            end: range.end,
            _pl: PhantomData,
        }
    }
}

impl<'de> Flavor<'de> for Slice<'de> {
    type Remainder = &'de [u8];
    type Source = &'de [u8];
    type PopError = UnexpectedEnd;
    type FinalizeError = Infallible;

    #[inline]
    fn pop(&mut self) -> Result<u8, Self::PopError> {
        if self.cursor == self.end {
            Err(UnexpectedEnd)
        } else {
            // SAFETY: `self.cursor` is in-bounds and won't be incremented past `self.end` as we
            // have checked above.
            unsafe {
                let res = Ok(*self.cursor);
                self.cursor = self.cursor.add(1);
                res
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some((self.end as usize) - (self.cursor as usize))
    }

    #[inline]
    fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8], Self::PopError> {
        let remain = (self.end as usize) - (self.cursor as usize);
        if remain < ct {
            Err(UnexpectedEnd)
        } else {
            // SAFETY: `self.cursor` is valid for `ct` elements and won't be incremented past `self.end` as we
            // have checked above.
            unsafe {
                let sli = core::slice::from_raw_parts(self.cursor, ct);
                self.cursor = self.cursor.add(ct);
                Ok(sli)
            }
        }
    }

    /// Return the remaining (unused) bytes in the Deserializer
    fn finalize(self) -> Result<&'de [u8], Infallible> {
        let remain = (self.end as usize) - (self.cursor as usize);
        // SAFETY: `self.cursor` is valid for `remain` elements
        unsafe { Ok(core::slice::from_raw_parts(self.cursor, remain)) }
    }
}

/// Support for [`std::io`]
#[cfg(feature = "std")]
pub mod io {
    use crate::de_flavors::UnexpectedEnd;
    use core::marker::PhantomData;

    struct SlidingBuffer<'de> {
        cursor: *mut u8,
        end: *const u8,
        _pl: PhantomData<&'de [u8]>,
    }

    impl<'de> SlidingBuffer<'de> {
        pub fn new(sli: &'de mut [u8]) -> Self {
            let range = sli.as_mut_ptr_range();
            Self {
                cursor: range.start,
                end: range.end,
                _pl: PhantomData,
            }
        }

        #[inline]
        fn take_n(&mut self, ct: usize) -> Result<&'de mut [u8], UnexpectedEnd> {
            let remain = (self.end as usize) - (self.cursor as usize);
            let buff = if remain < ct {
                return Err(UnexpectedEnd);
            } else {
                // SAFETY: `self.cursor` is valid for `ct` elements and won't be incremented
                // past `self.end` as we have checked above.
                unsafe {
                    let sli = core::slice::from_raw_parts_mut(self.cursor, ct);
                    self.cursor = self.cursor.add(ct);
                    sli
                }
            };

            Ok(buff)
        }

        #[inline]
        fn take_n_temp(&mut self, ct: usize) -> Result<&mut [u8], UnexpectedEnd> {
            let remain = (self.end as usize) - (self.cursor as usize);
            let buff = if remain < ct {
                return Err(UnexpectedEnd);
            } else {
                unsafe { core::slice::from_raw_parts_mut(self.cursor, ct) }
            };

            Ok(buff)
        }

        fn complete(self) -> &'de mut [u8] {
            let remain = (self.end as usize) - (self.cursor as usize);
            // SAFETY: `self.cursor` is valid for `remain` elements
            unsafe { core::slice::from_raw_parts_mut(self.cursor, remain) }
        }
    }

    /// Support for [`std::io`] traits
    #[allow(clippy::module_inception)]
    #[cfg(feature = "std")]
    pub mod io {
        use super::super::Flavor;
        use super::SlidingBuffer;
        use crate::de_flavors::UnexpectedEnd;

        /// Wrapper over a [`std::io::Read`] and a sliding buffer to implement the [Flavor] trait
        pub struct IOReader<'de, T>
        where
            T: std::io::Read,
        {
            reader: T,
            buff: SlidingBuffer<'de>,
        }

        impl<'de, T> IOReader<'de, T>
        where
            T: std::io::Read,
        {
            /// Create a new [`IOReader`] from a reader and a buffer.
            ///
            /// `buff` must have enough space to hold all data read during the deserialisation.
            pub fn new(reader: T, buff: &'de mut [u8]) -> Self {
                Self {
                    reader,
                    buff: SlidingBuffer::new(buff),
                }
            }
        }

        impl<'de, T> Flavor<'de> for IOReader<'de, T>
        where
            T: std::io::Read + 'de,
        {
            type Remainder = (T, &'de mut [u8]);
            type Source = &'de [u8];
            type PopError = UnexpectedEnd;
            type FinalizeError = core::convert::Infallible;

            #[inline]
            fn pop(&mut self) -> Result<u8, UnexpectedEnd> {
                let mut val = [0; 1];
                self.reader
                    .read_exact(&mut val)
                    .map_err(|_| UnexpectedEnd)?;
                Ok(val[0])
            }

            #[inline]
            fn size_hint(&self) -> Option<usize> {
                None
            }

            #[inline]
            fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8], UnexpectedEnd> {
                let buff = self.buff.take_n(ct)?;
                self.reader.read_exact(buff).map_err(|_| UnexpectedEnd)?;
                Ok(buff)
            }

            #[inline]
            fn try_take_n_temp<'a>(&'a mut self, ct: usize) -> Result<&'a [u8], UnexpectedEnd>
            where
                'de: 'a,
            {
                let buff = self.buff.take_n_temp(ct)?;
                self.reader.read_exact(buff).map_err(|_| UnexpectedEnd)?;
                Ok(buff)
            }

            /// Return the remaining (unused) bytes in the Deserializer
            fn finalize(self) -> Result<(T, &'de mut [u8]), core::convert::Infallible> {
                Ok((self.reader, self.buff.complete()))
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_pop() {
                let mut reader = IOReader::new(&[0xAA, 0xBB, 0xCC][..], &mut []);

                assert_eq!(reader.pop(), Ok(0xAA));
                assert_eq!(reader.pop(), Ok(0xBB));
                assert_eq!(reader.pop(), Ok(0xCC));
                assert_eq!(reader.pop(), Err(Error::DeserializeUnexpectedEnd));
            }

            #[test]
            fn test_try_take_n() {
                let mut buf = [0; 8];
                let mut reader = IOReader::new(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE][..], &mut buf);

                assert_eq!(reader.try_take_n(2), Ok(&[0xAA, 0xBB][..]));
                assert_eq!(reader.try_take_n(2), Ok(&[0xCC, 0xDD][..]));
                assert_eq!(reader.try_take_n(2), Err(Error::DeserializeUnexpectedEnd));
            }
        }
    }
}
