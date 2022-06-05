//! # Deserialization flavors
//!
//! Docs TO-DO!

use crate::{Result, Error};
use core::marker::PhantomData;

/// TODO
pub trait Flavor<'de>: 'de {
    /// TODO
    type Remainder: 'de;

    /// TODO
    fn pop(&mut self) -> Result<u8>;
    /// TODO
    fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8]>;
    /// TODO
    fn remaining(self) -> Result<Self::Remainder>;
}

/// TODO
pub struct Slice<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    pub(crate) cursor: *const u8,
    pub(crate) end: *const u8,
    pub(crate) _pl: PhantomData<&'de [u8]>,
}

impl<'de> Flavor<'de> for Slice<'de> {
    type Remainder = &'de [u8];

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
    fn remaining(self) -> Result<&'de [u8]> {
        let remain = (self.end as usize) - (self.cursor as usize);
        unsafe {
            Ok(core::slice::from_raw_parts(self.cursor, remain))
        }
    }
}
