//! # Flavors
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
//!     Slice { buf: buffer, idx: 0 }
//! ).unwrap();
//!
//! assert_eq!(res, &[0x04, 0x01, 0x00, 0x20, 0x30]);
//! ```
//!
//! ### Using combined flavors

use core::ops::Index;
use cobs::{EncoderState, PushResult};
use heapless::{ArrayLength, Vec};
use core::ops::IndexMut;

pub trait SerFlavor {
    type Output;

    fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
        data.iter()
            .try_for_each(|d| self.try_push(*d))
            .map_err(|_| ())
    }
    fn try_push(&mut self, data: u8) -> core::result::Result<(), ()>;
    fn release(self) -> core::result::Result<Self::Output, ()>;
}

pub struct Slice<'a> {
    pub buf: &'a mut [u8],
    pub idx: usize,
}

impl<'a> SerFlavor for Slice<'a> {
    type Output = &'a mut [u8];

    #[inline(always)]
    fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
        let len = data.len();

        if (len + self.idx) > self.buf.len() {
            return Err(());
        }

        self.buf[self.idx..self.idx + len]
            .copy_from_slice(data);

        self.idx += len;

        Ok(())
    }

    #[inline(always)]
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

pub struct HVec<B: ArrayLength<u8>>(Vec<u8, B>);

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

impl<B: ArrayLength<u8>> Default for HVec<B> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

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
    pub(crate) fn new(mut bee: B) -> Self {
        bee.try_push(0).unwrap();
        Self {
            flav: bee,
            cobs: EncoderState::default(),
        }
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
