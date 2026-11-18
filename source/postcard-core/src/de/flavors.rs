//! Basic deserialization flavors without deps outside of `core`/`alloc`/`std`.

use core::marker::PhantomData;

use super::{Flavor, UnexpectedEnd};

/// A simple [`Flavor`] representing the deserialization from a borrowed slice
pub struct Slice<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    cursor: *const u8,
    end: *const u8,
    _pl: PhantomData<&'de [u8]>,
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
    type PopError = UnexpectedEnd;
    type FinalizeError = core::convert::Infallible;

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
    fn finalize(self) -> Result<&'de [u8], core::convert::Infallible> {
        let remain = (self.end as usize) - (self.cursor as usize);
        // SAFETY: `self.cursor` is valid for `remain` elements
        unsafe { Ok(core::slice::from_raw_parts(self.cursor, remain)) }
    }
}
