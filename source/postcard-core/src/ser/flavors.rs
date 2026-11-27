//! Basic serialization flavors without deps outside of `core`/`alloc`/`std`.

use super::{BufferFull, Flavor, Patch};

/// The `Slice` flavor is a storage flavor, storing the serialized (or otherwise modified) bytes into a plain
/// `[u8]` slice. The `Slice` flavor resolves into a sub-slice of the original slice buffer.
pub struct Slice<'a> {
    buf: &'a mut [u8],
    // Safety Invariant: We must always ensure `pos <= buf.len()`.
    pos: usize,
}

impl<'a> Slice<'a> {
    /// Create a new `Slice` flavor from a given backing buffer
    pub fn new(buf: &'a mut [u8]) -> Self {
        Slice { buf, pos: 0 }
    }
}

impl<'a> Flavor for Slice<'a> {
    type Output = &'a mut [u8];
    type PushError = BufferFull;
    type FinalizeError = core::convert::Infallible;

    #[inline(always)]
    fn try_push(&mut self, b: u8) -> Result<(), BufferFull> {
        // SAFETY: We always check that self.pos is inbounds of buf. Perf
        // testing has shown that a slice + assert_unchecked optimizes better
        // than raw ptr math, because LLVM gains the knowledge that the buffer
        // is not aliased. This still requires the unsafe assert, but is a
        // reduction in the total unsafe code to audit.
        unsafe { core::hint::assert_unchecked(self.pos <= self.buf.len()) };
        match self.buf.get_mut(self.pos) {
            Some(slot) => {
                *slot = b;
                self.pos += 1;
                Ok(())
            }
            None => Err(BufferFull),
        }
    }

    #[inline(always)]
    fn try_extend(&mut self, b: &[u8]) -> Result<(), BufferFull> {
        // SAFETY: We always check that self.pos is inbounds of buf. See
        // `try_push` for more details.
        unsafe { core::hint::assert_unchecked(self.pos <= self.buf.len()) };
        // Perf note: slicing `pos..` then `..len` avoids an overflow check on
        // `pos + len`.
        let dst = self
            .buf
            .get_mut(self.pos..)
            .and_then(|r| r.get_mut(..b.len()));
        match dst {
            Some(dst) => {
                dst.copy_from_slice(b);
                self.pos += b.len();
                Ok(())
            }
            None => Err(BufferFull),
        }
    }

    fn finalize(self) -> Result<Self::Output, core::convert::Infallible> {
        let Slice { buf, pos } = self;
        // SAFETY: We always check that self.pos is inbounds of buf. See
        // `try_push` for more details.
        unsafe { core::hint::assert_unchecked(pos <= buf.len()) };
        Ok(&mut buf[..pos])
    }
}

impl Patch for Slice<'_> {
    #[inline]
    fn written_mut(&mut self) -> &mut [u8] {
        // SAFETY: We always check that self.pos is inbounds of buf. See
        // `try_push` for more details.
        unsafe { core::hint::assert_unchecked(self.pos <= self.buf.len()) };
        &mut self.buf[..self.pos]
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
