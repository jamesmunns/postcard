//! # Serialization Flavors
//!
//! "Flavors" in `postcard` are used as modifiers to the serialization or deserialization
//! process. Flavors typically modify one or both of the following:
//!
//! 1. The output medium of the serialization, e.g. whether the data is serialized to a `[u8]` slice, or a `heapless::Vec`.
//! 2. The format of the serialization, such as encoding the serialized output in a COBS format, performing CRC32 checksumming while serializing, etc.
//!
//! Flavors are implemented using the [`Flavor`] trait, which acts as a "middleware" for receiving the bytes as serialized by `serde`.
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
//! ## When to use (multiple) flavors
//!
//! Combining flavors are nice for convenience, as they perform potentially multiple steps of
//! serialization at one time.
//!
//! This can often be more memory efficient, as intermediate buffers are not typically required.
//!
//! ## When NOT to use (multiple) flavors
//!
//! The downside of passing serialization through multiple steps is that it is typically slower than
//! performing each step serially. Said simply, "cobs encoding while serializing" is often slower
//! than "serialize then cobs encode", due to the ability to handle longer "runs" of data in each
//! stage. The downside is that if these stages can not be performed in-place on the buffer, you
//! will need additional buffers for each stage.
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
//!     ser_flavors::Slice,
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
//!     ser_flavors::{Cobs, Slice},
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
use cobs::{EncoderState, PushResult};
use core::marker::PhantomData;
use core::ops::Index;
use core::ops::IndexMut;

#[cfg(feature = "heapless")]
pub use heapless_vec::*;

#[cfg(feature = "use-std")]
pub use std_vec::*;

#[cfg(feature = "alloc")]
pub use alloc_vec::*;

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

    /// The try_extend() trait method can be implemented when there is a more efficient way of processing
    /// multiple bytes at once, such as copying a slice to the output, rather than iterating over one byte
    /// at a time.
    #[inline]
    fn try_extend(&mut self, data: &[u8]) -> Result<()> {
        data.iter().try_for_each(|d| self.try_push(*d))
    }

    /// The try_push() trait method can be used to push a single byte to be modified and/or stored
    fn try_push(&mut self, data: u8) -> Result<()>;

    /// Finalize the serialization process
    fn finalize(self) -> Result<Self::Output>;
}

////////////////////////////////////////
// Slice
////////////////////////////////////////

/// The `Slice` flavor is a storage flavor, storing the serialized (or otherwise modified) bytes into a plain
/// `[u8]` slice. The `Slice` flavor resolves into a sub-slice of the original slice buffer.
pub struct Slice<'a> {
    start: *mut u8,
    cursor: *mut u8,
    end: *mut u8,
    _pl: PhantomData<&'a [u8]>,
}

impl<'a> Slice<'a> {
    /// Create a new `Slice` flavor from a given backing buffer
    pub fn new(buf: &'a mut [u8]) -> Self {
        Slice {
            start: buf.as_mut_ptr(),
            cursor: buf.as_mut_ptr(),
            end: unsafe { buf.as_mut_ptr().add(buf.len()) },
            _pl: PhantomData,
        }
    }
}

impl<'a> Flavor for Slice<'a> {
    type Output = &'a mut [u8];

    #[inline(always)]
    fn try_push(&mut self, b: u8) -> Result<()> {
        if self.cursor == self.end {
            Err(Error::SerializeBufferFull)
        } else {
            unsafe {
                self.cursor.write(b);
                self.cursor = self.cursor.add(1);
            }
            Ok(())
        }
    }

    #[inline(always)]
    fn try_extend(&mut self, b: &[u8]) -> Result<()> {
        let remain = (self.end as usize) - (self.cursor as usize);
        let blen = b.len();
        if blen > remain {
            Err(Error::SerializeBufferFull)
        } else {
            unsafe {
                core::ptr::copy_nonoverlapping(b.as_ptr(), self.cursor, blen);
                self.cursor = self.cursor.add(blen);
            }
            Ok(())
        }
    }

    fn finalize(self) -> Result<Self::Output> {
        let used = (self.cursor as usize) - (self.start as usize);
        let sli = unsafe { core::slice::from_raw_parts_mut(self.start, used) };
        Ok(sli)
    }
}

impl<'a> Index<usize> for Slice<'a> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        let len = (self.end as usize) - (self.start as usize);
        assert!(idx < len);
        unsafe { &*self.start.add(idx) }
    }
}

impl<'a> IndexMut<usize> for Slice<'a> {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        let len = (self.end as usize) - (self.start as usize);
        assert!(idx < len);
        unsafe { &mut *self.start.add(idx) }
    }
}

#[cfg(feature = "heapless")]
mod heapless_vec {
    use super::Flavor;
    use super::Index;
    use super::IndexMut;
    use crate::{Error, Result};
    use heapless::Vec;

    ////////////////////////////////////////
    // HVec
    ////////////////////////////////////////

    /// The `HVec` flavor is a wrapper type around a `heapless::Vec`. This is a stack
    /// allocated data structure, with a fixed maximum size and variable amount of contents.
    #[derive(Default)]
    pub struct HVec<const B: usize> {
        /// the contained data buffer
        vec: Vec<u8, B>,
    }

    impl<const B: usize> HVec<B> {
        /// Create a new, currently empty, [heapless::Vec] to be used for storing serialized
        /// output data.
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl<'a, const B: usize> Flavor for HVec<B> {
        type Output = Vec<u8, B>;

        #[inline(always)]
        fn try_extend(&mut self, data: &[u8]) -> Result<()> {
            self.vec
                .extend_from_slice(data)
                .map_err(|_| Error::SerializeBufferFull)
        }

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> Result<()> {
            self.vec.push(data).map_err(|_| Error::SerializeBufferFull)
        }

        fn finalize(self) -> Result<Vec<u8, B>> {
            Ok(self.vec)
        }
    }

    impl<const B: usize> Index<usize> for HVec<B> {
        type Output = u8;

        fn index(&self, idx: usize) -> &u8 {
            &self.vec[idx]
        }
    }

    impl<const B: usize> IndexMut<usize> for HVec<B> {
        fn index_mut(&mut self, idx: usize) -> &mut u8 {
            &mut self.vec[idx]
        }
    }
}

#[cfg(feature = "use-std")]
mod std_vec {
    /// The `StdVec` flavor is a wrapper type around a `std::vec::Vec`.
    ///
    /// This type is only available when the (non-default) `use-std` feature is active
    pub type StdVec = super::alloc_vec::AllocVec;
}

#[cfg(feature = "alloc")]
mod alloc_vec {
    extern crate alloc;
    use super::Flavor;
    use super::Index;
    use super::IndexMut;
    use crate::Result;
    use alloc::vec::Vec;

    /// The `AllocVec` flavor is a wrapper type around an [alloc::vec::Vec].
    ///
    /// This type is only available when the (non-default) `alloc` feature is active
    #[derive(Default)]
    pub struct AllocVec {
        /// The vec to be used for serialization
        vec: Vec<u8>,
    }

    impl AllocVec {
        /// Create a new, currently empty, [alloc::vec::Vec] to be used for storing serialized
        /// output data.
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Flavor for AllocVec {
        type Output = Vec<u8>;

        #[inline(always)]
        fn try_extend(&mut self, data: &[u8]) -> Result<()> {
            self.vec.extend_from_slice(data);
            Ok(())
        }

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> Result<()> {
            self.vec.push(data);
            Ok(())
        }

        fn finalize(self) -> Result<Self::Output> {
            Ok(self.vec)
        }
    }

    impl Index<usize> for AllocVec {
        type Output = u8;

        #[inline]
        fn index(&self, idx: usize) -> &u8 {
            &self.vec[idx]
        }
    }

    impl IndexMut<usize> for AllocVec {
        #[inline]
        fn index_mut(&mut self, idx: usize) -> &mut u8 {
            &mut self.vec[idx]
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
    B: Flavor + IndexMut<usize, Output = u8>,
{
    flav: B,
    cobs: EncoderState,
}

impl<B> Cobs<B>
where
    B: Flavor + IndexMut<usize, Output = u8>,
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

impl<'a, B> Flavor for Cobs<B>
where
    B: Flavor + IndexMut<usize, Output = u8>,
{
    type Output = <B as Flavor>::Output;

    #[inline(always)]
    fn try_push(&mut self, data: u8) -> Result<()> {
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

    fn finalize(mut self) -> Result<Self::Output> {
        let (idx, mval) = self.cobs.finalize();
        self.flav[idx] = mval;
        self.flav.try_push(0)?;
        self.flav.finalize()
    }
}
