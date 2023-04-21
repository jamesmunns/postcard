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

////////////////////////////////////////
// CRC
////////////////////////////////////////

/// This Cyclic Redundancy Check flavor applies [the CRC crate's `Algorithm`](https://docs.rs/crc/latest/crc/struct.Algorithm.html) struct on
/// the serialized data. The flavor will check the CRC assuming that it has been appended to the bytes.
///
/// CRCs are used for error detection when reading data back.
///
/// The `crc` feature requires enabling to use this module.
///
/// More on CRCs: <https://en.wikipedia.org/wiki/Cyclic_redundancy_check>.
#[cfg(feature = "use-crc")]
pub mod crc {
    use core::convert::TryInto;

    use crc::Digest;
    use crc::Width;
    use serde::Deserialize;

    use self::sealed::CrcAble;

    use super::Flavor;
    use super::Slice;

    use crate::Deserializer;
    use crate::Error;
    use crate::Result;

    mod sealed {
        use core::convert::TryFrom;

        use crc::{Width, Digest};

        pub trait CrcAble: Width + PartialEq {
            const NUM_BYTES: usize;
            type BYTES: for<'a> TryFrom<&'a [u8]>;
            const _CHECK: Self::BYTES;

            fn from_bytes(bytes: Self::BYTES) -> Self;
            fn update_fun<'a, 'b, 'c>(d: &'a mut Digest<'b, Self>, b: &'c [u8]);
            fn final_fun<'a>(d: Digest<'a, Self>) -> Self;
        }

        macro_rules! impl_crcable {
            ($( $int:ty ),*) => {
                $(
                    impl CrcAble for $int {
                        const NUM_BYTES: usize = core::mem::size_of::<$int>();
                        type BYTES = [u8; core::mem::size_of::<$int>()];
                        const _CHECK: Self::BYTES = [0u8; Self::NUM_BYTES];

                        #[inline(always)]
                        fn update_fun<'a, 'b, 'c>(d: &'a mut Digest<'b, Self>, b: &'c [u8]) {
                            Digest::<'b, Self>::update(d, b);
                        }

                        #[inline(always)]
                        fn final_fun<'a>(d: Digest<'a, Self>) -> Self {
                            Digest::<'a, Self>::finalize(d)
                        }

                        #[inline(always)]
                        fn from_bytes(bytes: Self::BYTES) -> Self {
                            <$int>::from_le_bytes(bytes)
                        }
                    }
                )*
            };
        }

        impl_crcable!(u8, u16, u32, u64, u128);

    }


    /// Manages CRC modifications as a flavor.
    pub struct CrcModifier<'de, B, W>
    where
        B: Flavor<'de>,
        W: Width,
    {
        flav: B,
        digest: Digest<'de, W>,
    }

    impl<'de, B, W> CrcModifier<'de, B, W>
    where
        B: Flavor<'de>,
        W: Width,
    {
        /// Create a new Crc modifier Flavor.
        pub fn new(bee: B, digest: Digest<'de, W>) -> Self {
            Self { flav: bee, digest }
        }
    }


    impl<'de, B, W> Flavor<'de> for CrcModifier<'de, B, W>
    where
        B: Flavor<'de>,
        W: CrcAble,
    {
        type Remainder = B::Remainder;

        type Source = B::Source;

        #[inline]
        fn pop(&mut self) -> Result<u8> {
            match self.flav.pop() {
                Ok(byte) => {
                    W::update_fun(&mut self.digest, &[byte]);
                    Ok(byte)
                }
                e @ Err(_) => e,
            }
        }

        #[inline]
        fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8]> {
            match self.flav.try_take_n(ct) {
                Ok(bytes) => {
                    W::update_fun(&mut self.digest, bytes);
                    Ok(bytes)
                }
                e @ Err(_) => e,
            }
        }

        fn finalize(mut self) -> Result<Self::Remainder> {
            match self.flav.try_take_n(core::mem::size_of::<W>()) {
                Ok(prev_crc_bytes) => match self.flav.finalize() {
                    Ok(remainder) => {
                        let crc = W::final_fun(self.digest);
                        let le_bytes: W::BYTES = prev_crc_bytes
                            .try_into()
                            .map_err(|_| Error::DeserializeBadEncoding)?;
                        let prev_crc = <W>::from_bytes(le_bytes);
                        if crc == prev_crc {
                            Ok(remainder)
                        } else {
                            Err(Error::DeserializeBadEncoding)
                        }
                    }
                    e @ Err(_) => e,
                },
                Err(e) => Err(e),
            }
        }
    }

    /// Deserialize a message of type `T` from a byte slice with a Crc. The unused portion (if any)
    /// of the byte slice is not returned.
    pub fn from_bytes_width<'a, T, W>(s: &'a [u8], digest: Digest<'a, W>) -> Result<T>
    where
        T: Deserialize<'a>,
        W: CrcAble,
    {
        let flav = CrcModifier::new(Slice::new(s), digest);
        let mut deserializer = Deserializer::from_flavor(flav);
        let r = T::deserialize(&mut deserializer)?;
        let _ = deserializer.finalize()?;
        Ok(r)
    }

    /// Deserialize a message of type `T` from a byte slice with a Crc. The unused portion (if any)
    /// of the byte slice is returned for further usage
    pub fn from_bytes_u32<'a, T>(s: &'a [u8], digest: Digest<'a, u32>) -> Result<T>
    where
        T: Deserialize<'a>,
    {
        from_bytes_width(s, digest)
    }

    /// Deserialize a message of type `T` from a byte slice with a Crc. The unused portion (if any)
    /// of the byte slice is returned for further usage
    pub fn take_from_bytes_width<'a, T, W>(s: &'a [u8], digest: Digest<'a, W>) -> Result<(T, &'a [u8])>
    where
        T: Deserialize<'a>,
        W: CrcAble,
    {
        let flav = CrcModifier::new(Slice::new(s), digest);
        let mut deserializer = Deserializer::from_flavor(flav);
        let t = T::deserialize(&mut deserializer)?;
        Ok((t, deserializer.finalize()?))
    }

    /// Deserialize a message of type `T` from a byte slice with a Crc. The unused portion (if any)
    /// of the byte slice is returned for further usage
    pub fn take_from_bytes_u32<'a, T>(s: &'a [u8], digest: Digest<'a, u32>) -> Result<(T, &'a [u8])>
    where
        T: Deserialize<'a>,
    {
        take_from_bytes_width(s, digest)
    }
}
