//! # Flavors - Plugins for `postcard`
//!
//! "Flavors" in `postcard` are used as modifiers to the serialization
//! process. Flavors typically modify one or both of the following:
//!
//! 1. The output medium of the serialization, e.g. whether the data is serialized to a `[u8]` slice, or a `heapless::Vec`.
//! 2. The format of the serialization, such as encoding the serialized output in a COBS format, performing CRC32 checksumming while serializing, etc.
//!
//! Flavors are implemented using the `SerFlavor` trait, which acts as a "middleware" for receiving the bytes as serialized by `serde`.
//! Multiple flavors may be combined to obtain a desired combination of behavior and storage.
//! When flavors are combined, it is expected that the storage flavor (such as `Slice` or `HVec`) is the innermost flavor.
//!
//! Custom flavors may be defined by users of the `postcard` crate, however some commonly useful flavors have been provided in
//! this module. If you think your custom flavor would be useful to others, PRs adding flavors are very welcome!
//!
//! ## Usability
//!
//! Flavors may not always be convenient to use directly, as they may expose some implementation details of how the
//! inner workings of the flavor behaves. It is typical to provide a convenience method for using a flavor, to prevent
//! the user from having to specify generic parameters, setting correct initialization values, or handling the output of
//! the flavor correctly. See `postcard::to_vec()` for an example of this.
//!
//! It is recommended to use the [`serialize_with_flavor()`](../fn.serialize_with_flavor.html) method for serialization. See it's documentation for information
//! regarding usage and generic type parameters.
//!
//! ## Examples
//!
//! ### Using a single flavor
//!
//! In the first example, we use the `Slice` flavor, to store the serialized output into a mutable `[u8]` slice.
//! No other modification is made to the serialization process.
//!
//! ```rust
//! use postcard::{
//!     serialize_with_flavor,
//!     flavors::Slice,
//! };
//!
//! let mut buf = [0u8; 32];
//!
//! let data: &[u8] = &[0x01, 0x00, 0x20, 0x30];
//! let buffer = &mut [0u8; 32];
//! let res = serialize_with_flavor::<[u8], Slice, &mut [u8]>(
//!     data,
//!     Slice::new(buffer)
//! ).unwrap();
//!
//! assert_eq!(res, &[0x04, 0x01, 0x00, 0x20, 0x30]);
//! ```
//!
//! ### Using combined flavors
//!
//! In the second example, we mix `Slice` with `Cobs`, to cobs encode the output while
//! the data is serialized. Notice how `Slice` (the storage flavor) is the innermost flavor used.
//!
//! ```rust
//! use postcard::{
//!     serialize_with_flavor,
//!     flavors::{Cobs, Slice},
//! };
//!
//! let mut buf = [0u8; 32];
//!
//! let data: &[u8] = &[0x01, 0x00, 0x20, 0x30];
//! let buffer = &mut [0u8; 32];
//! let res = serialize_with_flavor::<[u8], Cobs<Slice>, &mut [u8]>(
//!     data,
//!     Cobs::try_new(Slice::new(buffer)).unwrap(),
//! ).unwrap();
//!
//! assert_eq!(res, &[0x03, 0x04, 0x01, 0x03, 0x20, 0x30, 0x00]);
//! ```

use crate::error::{Error, Result};
use crate::varint::VarintUsize;
use cobs::{EncoderState, PushResult};
use core::ops::Index;
use core::ops::IndexMut;

#[cfg(feature = "heapless")]
pub use heapless_vec::*;

#[cfg(feature = "use-std")]
pub use std_vec::*;

#[cfg(feature = "alloc")]
pub use alloc_vec::*;

/// The SerFlavor trait acts as a combinator/middleware interface that can be used to pass bytes
/// through storage or modification flavors. See the module level documentation for more information
/// and examples.
pub trait SerFlavor {
    /// The `Output` type is what this flavor "resolves" to when the serialization is complete.
    /// For storage flavors, this is typically a concrete type. For modification flavors, this is
    /// typically a generic parameter for the storage flavor they are wrapped around.
    type Output;

    /// The try_extend() trait method can be implemented when there is a more efficient way of processing
    /// multiple bytes at once, such as copying a slice to the output, rather than iterating over one byte
    /// at a time.
    fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
        data.iter()
            .try_for_each(|d| self.try_push(*d))
            .map_err(|_| ())
    }

    /// The try_push() trait method can be used to push a single byte to be modified and/or stored
    fn try_push(&mut self, data: u8) -> core::result::Result<(), ()>;

    /// The try_push_varint_usize() trait method can be used to push a `VarintUsize`. The default
    /// implementation uses try_extend() to process the encoded `VarintUsize` bytes, which is likely
    /// the desired behavior for most circumstances.
    fn try_push_varint_usize(&mut self, data: &VarintUsize) -> core::result::Result<(), ()> {
        let mut buf = VarintUsize::new_buf();
        let used_buf = data.to_buf(&mut buf);
        self.try_extend(used_buf)
    }

    /// The release() trait method finalizes the modification or storage operation, and resolved into
    /// the type defined by `SerFlavor::Output` associated type.
    fn release(self) -> core::result::Result<Self::Output, ()>;
}

////////////////////////////////////////////////////////////////////////////////
// Storage Flavors
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////
// Slice
////////////////////////////////////////

/// The `Slice` flavor is a storage flavor, storing the serialized (or otherwise modified) bytes into a plain
/// `[u8]` slice. The `Slice` flavor resolves into a sub-slice of the original slice buffer.
pub struct Slice<'a> {
    buf: &'a mut [u8],
    idx: usize,
}

impl<'a> Slice<'a> {
    /// Create a new `Slice` flavor from a given backing buffer
    pub fn new(buf: &'a mut [u8]) -> Self {
        Slice { buf, idx: 0 }
    }
}

impl<'a> SerFlavor for Slice<'a> {
    type Output = &'a mut [u8];

    fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
        let len = data.len();

        if (len + self.idx) > self.buf.len() {
            return Err(());
        }

        self.buf[self.idx..self.idx + len].copy_from_slice(data);

        self.idx += len;

        Ok(())
    }

    fn try_push(&mut self, data: u8) -> core::result::Result<(), ()> {
        if self.idx >= self.buf.len() {
            return Err(());
        }

        self.buf[self.idx] = data;
        self.idx += 1;

        Ok(())
    }

    fn release(self) -> core::result::Result<Self::Output, ()> {
        let (used, _unused) = self.buf.split_at_mut(self.idx);
        Ok(used)
    }
}

impl<'a> Index<usize> for Slice<'a> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.buf[idx]
    }
}

impl<'a> IndexMut<usize> for Slice<'a> {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.buf[idx]
    }
}

#[cfg(feature = "heapless")]
mod heapless_vec {
    use heapless::{ArrayLength, Vec};
    use super::SerFlavor;
    use super::Index;
    use super::IndexMut;

    ////////////////////////////////////////
    // HVec
    ////////////////////////////////////////

    /// The `HVec` flavor is a wrapper type around a `heapless::Vec`. This is a stack
    /// allocated data structure, with a fixed maximum size and variable amount of contents.
    pub struct HVec<B: ArrayLength<u8>>(Vec<u8, B>);

    impl<'a, B> SerFlavor for HVec<B>
    where
        B: ArrayLength<u8>,
    {
        type Output = Vec<u8, B>;

        #[inline(always)]
        fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
            self.0.extend_from_slice(data)
        }

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> core::result::Result<(), ()> {
            self.0.push(data).map_err(|_| ())
        }

        fn release(self) -> core::result::Result<Vec<u8, B>, ()> {
            Ok(self.0)
        }
    }

    impl<B: ArrayLength<u8>> Index<usize> for HVec<B> {
        type Output = u8;

        fn index(&self, idx: usize) -> &u8 {
            &self.0[idx]
        }
    }

    impl<B: ArrayLength<u8>> IndexMut<usize> for HVec<B> {
        fn index_mut(&mut self, idx: usize) -> &mut u8 {
            &mut self.0[idx]
        }
    }

    impl<B: ArrayLength<u8>> Default for HVec<B> {
        fn default() -> Self {
            Self(Vec::new())
        }
    }
}

#[cfg(feature = "use-std")]
mod std_vec {
    extern crate std;
    use std::vec::Vec;
    use super::SerFlavor;
    use super::Index;
    use super::IndexMut;

    /// The `StdVec` flavor is a wrapper type around a `std::vec::Vec`.
    ///
    /// This type is only available when the (non-default) `use-std` feature is active
    pub struct StdVec(pub Vec<u8>);

    impl SerFlavor for StdVec {
        type Output = Vec<u8>;

        #[inline(always)]
        fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
            self.0.extend_from_slice(data);
            Ok(())
        }

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> core::result::Result<(), ()> {
            self.0.push(data);
            Ok(())
        }

        fn release(self) -> core::result::Result<Self::Output, ()> {
            Ok(self.0)
        }
    }

    impl Index<usize> for StdVec {
        type Output = u8;

        fn index(&self, idx: usize) -> &u8 {
            &self.0[idx]
        }
    }

    impl IndexMut<usize> for StdVec {
        fn index_mut(&mut self, idx: usize) -> &mut u8 {
            &mut self.0[idx]
        }
    }
}

#[cfg(feature = "alloc")]
mod alloc_vec {
    extern crate alloc;
    use alloc::vec::Vec;
    use super::SerFlavor;
    use super::Index;
    use super::IndexMut;

    /// The `AllocVec` flavor is a wrapper type around an `alloc::vec::Vec`.
    ///
    /// This type is only available when the (non-default) `alloc` feature is active
    pub struct AllocVec(pub Vec<u8>);

    impl SerFlavor for AllocVec {
        type Output = Vec<u8>;

        #[inline(always)]
        fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
            self.0.extend_from_slice(data);
            Ok(())
        }

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> core::result::Result<(), ()> {
            self.0.push(data);
            Ok(())
        }

        fn release(self) -> core::result::Result<Self::Output, ()> {
            Ok(self.0)
        }
    }

    impl Index<usize> for AllocVec {
        type Output = u8;

        fn index(&self, idx: usize) -> &u8 {
            &self.0[idx]
        }
    }

    impl IndexMut<usize> for AllocVec {
        fn index_mut(&mut self, idx: usize) -> &mut u8 {
            &mut self.0[idx]
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Modification Flavors
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////
// COBS
////////////////////////////////////////

/// The `Cobs` flavor implements [Consistent Overhead Byte Stuffing] on
/// the serialized data. The output of this flavor includes the termination/sentinel
/// byte of `0x00`.
///
/// This protocol is useful when sending data over a serial interface without framing such as a UART
///
/// [Consistent Overhead Byte Stuffing]: https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing
pub struct Cobs<B>
where
    B: SerFlavor + IndexMut<usize, Output = u8>,
{
    flav: B,
    cobs: EncoderState,
}

impl<B> Cobs<B>
where
    B: SerFlavor + IndexMut<usize, Output = u8>,
{
    /// Create a new Cobs modifier Flavor. If there is insufficient space
    /// to push the leading header byte, the method will return an Error
    pub fn try_new(mut bee: B) -> Result<Self> {
        bee.try_push(0).map_err(|_| Error::SerializeBufferFull)?;
        Ok(Self {
            flav: bee,
            cobs: EncoderState::default(),
        })
    }
}

impl<'a, B> SerFlavor for Cobs<B>
where
    B: SerFlavor + IndexMut<usize, Output = u8>,
{
    type Output = <B as SerFlavor>::Output;

    #[inline(always)]
    fn try_push(&mut self, data: u8) -> core::result::Result<(), ()> {
        use PushResult::*;
        match self.cobs.push(data) {
            AddSingle(n) => self.flav.try_push(n),
            ModifyFromStartAndSkip((idx, mval)) => {
                self.flav[idx] = mval;
                self.flav.try_push(0)
            }
            ModifyFromStartAndPushAndSkip((idx, mval, nval)) => {
                self.flav[idx] = mval;
                self.flav.try_push(nval)?;
                self.flav.try_push(0)
            }
        }
    }

    fn release(mut self) -> core::result::Result<Self::Output, ()> {
        let (idx, mval) = self.cobs.finalize();
        self.flav[idx] = mval;
        self.flav.try_push(0)?;
        self.flav.release()
    }
}
