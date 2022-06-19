//! # Deserialization flavors
//!
//! Docs TO-DO!

use crate::{Error, Result};
use core::marker::PhantomData;

/// TODO
pub trait Flavor<'de>: 'de {
    /// TODO
    type Remainder: 'de;
    /// TODO
    type Source: 'de;

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

impl<'de> Slice<'de> {
    /// TODO
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
    fn remaining(self) -> Result<&'de [u8]> {
        let remain = (self.end as usize) - (self.cursor as usize);
        unsafe { Ok(core::slice::from_raw_parts(self.cursor, remain)) }
    }
}

// Write a terrible checksum implementation to make sure that we can effectively
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
        fn remaining(self) -> Result<Self::Remainder> {
            Ok((self.flav.remaining()?, self.checksum))
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
        let (rem, cksm) = deser.remaining().unwrap();

        // The pre-calculated checksum we stuffed at the end is the
        // first "unused" byte
        assert_eq!(rem[0], check);

        // the one we calculated during serialization matches the
        // pre-calculated one
        assert_eq!(cksm, check);
    }
}
