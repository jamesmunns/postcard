//! Basic serialization flavors without deps outside of `core`/`alloc`/`std`.

use core::marker::PhantomData;

use super::{BufferFull, Flavor, Patch};

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
        let ptr = buf.as_mut_ptr_range();
        Slice {
            start: ptr.start,
            cursor: ptr.start,
            end: ptr.end,
            _pl: PhantomData,
        }
    }
}

impl<'a> Flavor for Slice<'a> {
    type Output = &'a mut [u8];
    type PushError = BufferFull;
    type FinalizeError = core::convert::Infallible;

    #[inline(always)]
    fn try_push(&mut self, b: u8) -> Result<(), BufferFull> {
        if self.cursor == self.end {
            Err(BufferFull)
        } else {
            // SAFETY: `self.cursor` is in-bounds and won't be incremented past `self.end` as we
            // have checked above.
            unsafe {
                self.cursor.write(b);
                self.cursor = self.cursor.add(1);
            }
            Ok(())
        }
    }

    #[inline(always)]
    fn try_extend(&mut self, b: &[u8]) -> Result<(), BufferFull> {
        let remain = (self.end as usize) - (self.cursor as usize);
        let blen = b.len();
        if blen > remain {
            Err(BufferFull)
        } else {
            // SAFETY: `self.cursor` is in-bounds for `blen` elements and won't be incremented past
            // `self.end` as we have checked above.
            unsafe {
                core::ptr::copy_nonoverlapping(b.as_ptr(), self.cursor, blen);
                self.cursor = self.cursor.add(blen);
            }
            Ok(())
        }
    }

    fn finalize(self) -> Result<Self::Output, core::convert::Infallible> {
        let used = (self.cursor as usize) - (self.start as usize);
        // SAFETY: `self.cursor` is in-bounds for `used` elements
        let sli = unsafe { core::slice::from_raw_parts_mut(self.start, used) };
        Ok(sli)
    }
}

impl Patch for Slice<'_> {
    #[inline]
    fn written_mut(&mut self) -> &mut [u8] {
        let used = (self.cursor as usize) - (self.start as usize);
        // SAFETY: `start..cursor` is within the buffer given to `new`, which
        // this flavor borrows mutably for `'a`, and the returned slice borrows
        // `self` mutably, so nothing else can access those bytes while it lives.
        unsafe { core::slice::from_raw_parts_mut(self.start, used) }
    }
}

/// The `Size` flavor is a measurement flavor, which accumulates the number of bytes needed to
/// serialize the data.
///
/// ```
/// use postcard_core::ser::{self, Flavor, flavors::Size};
///
/// let mut size = Size::default();
/// ser::try_push_bool(&mut size, false).unwrap();
/// ser::try_push_str(&mut size, "Hi!").unwrap();
///
/// assert_eq!(size.finalize().unwrap(), 5);
/// ```
#[derive(Default)]
pub struct Size {
    size: usize,
}

impl Flavor for Size {
    type Output = usize;
    type PushError = core::convert::Infallible;
    type FinalizeError = core::convert::Infallible;

    #[inline(always)]
    fn try_push(&mut self, _b: u8) -> Result<(), core::convert::Infallible> {
        self.size += 1;
        Ok(())
    }

    #[inline(always)]
    fn try_extend(&mut self, b: &[u8]) -> Result<(), core::convert::Infallible> {
        self.size += b.len();
        Ok(())
    }

    fn finalize(self) -> Result<Self::Output, core::convert::Infallible> {
        Ok(self.size)
    }
}

/// Wrapper over a [`core::iter::Extend<u8>`] that implements the flavor trait
pub struct ExtendFlavor<T> {
    iter: T,
}

impl<T> ExtendFlavor<T>
where
    T: core::iter::Extend<u8>,
{
    /// Create a new [`Self`] flavor from a given [`core::iter::Extend<u8>`]
    pub fn new(iter: T) -> Self {
        Self { iter }
    }
}

impl<T> Flavor for ExtendFlavor<T>
where
    T: core::iter::Extend<u8>,
{
    type Output = T;
    type PushError = core::convert::Infallible;
    type FinalizeError = core::convert::Infallible;

    #[inline(always)]
    fn try_push(&mut self, data: u8) -> Result<(), core::convert::Infallible> {
        self.iter.extend([data]);
        Ok(())
    }

    #[inline(always)]
    fn try_extend(&mut self, b: &[u8]) -> Result<(), core::convert::Infallible> {
        self.iter.extend(b.iter().copied());
        Ok(())
    }

    fn finalize(self) -> Result<Self::Output, core::convert::Infallible> {
        Ok(self.iter)
    }
}

// TODO: In the future we will generalize to `impl<A: Allocator> ... for Vec<u8, A>`,
// once the MSRV is at least 1.100. This is not expected to be a breaking change.
#[cfg(feature = "alloc")]
mod alloc_vec {
    use alloc::vec::Vec;

    use crate::ser::{Flavor, Patch};

    /// Appends to the `Vec`, and returns it from [`Flavor::finalize`]
    ///
    /// NOTE: Use `&mut Vec<u8>` to append to a `Vec` without moving ownership.
    impl Flavor for Vec<u8> {
        type Output = Vec<u8>;
        type PushError = core::convert::Infallible;
        type FinalizeError = core::convert::Infallible;

        #[inline]
        fn try_extend(&mut self, data: &[u8]) -> Result<(), Self::PushError> {
            self.extend_from_slice(data);
            Ok(())
        }

        #[inline]
        fn try_push(&mut self, data: u8) -> Result<(), Self::PushError> {
            self.push(data);
            Ok(())
        }

        #[inline]
        fn finalize(self) -> Result<Self::Output, Self::FinalizeError> {
            Ok(self)
        }
    }

    /// Returns the whole `Vec`, including any bytes it held before serialization
    /// started.
    impl Patch for Vec<u8> {
        #[inline]
        fn written_mut(&mut self) -> &mut [u8] {
            self
        }
    }
}
