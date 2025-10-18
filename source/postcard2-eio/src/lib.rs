use postcard2::{serialize_with_flavor, Deserializer};
use serde_core::{Deserialize, Serialize};

/// Serialize a `T` to an [`embedded_io Write`](embedded_io::Write),
/// ## Example
///
/// ```rust
/// use postcard2_eio::to_eio;
/// let mut buf: [u8; 32] = [0; 32];
/// let mut writer: &mut [u8] = &mut buf;
///
/// let ser = to_eio(&true, &mut writer).unwrap();
/// to_eio("Hi!", ser).unwrap();
/// assert_eq!(&buf[0..5], &[0x01, 0x03, b'H', b'i', b'!']);
/// ```
pub fn to_eio<T, W>(value: &T, writer: W) -> postcard2::Result<W>
where
    T: Serialize + ?Sized,
    W: embedded_io::Write,
{
    serialize_with_flavor::<T, _, _>(value, ser::WriteFlavor::new(writer))
}

/// Deserialize a message of type `T` from a [`embedded_io`](embedded_io::embedded_io)::[`Read`](embedded_io::Read).
pub fn from_eio<'a, T, R>(val: (R, &'a mut [u8])) -> postcard2::Result<(T, (R, &'a mut [u8]))>
where
    T: Deserialize<'a>,
    R: embedded_io::Read + 'a,
{
    let flavor = de::EIOReader::new(val.0, val.1);
    let mut deserializer = Deserializer::from_flavor(flavor);
    let t = T::deserialize(&mut deserializer)?;
    Ok((t, deserializer.finalize()?))
}

pub mod ser {
    use postcard2::ser_flavors::Flavor;
    use postcard2::{Error, Result};

    /// Wrapper over a [`embedded_io Write`](embedded_io::Write) that implements the flavor trait
    pub struct WriteFlavor<T> {
        writer: T,
    }

    impl<T> WriteFlavor<T>
    where
        T: embedded_io::Write,
    {
        /// Create a new [`Self`] flavor from a given [`embedded_io Write`](embedded_io::Write)
        pub fn new(writer: T) -> Self {
            Self { writer }
        }
    }

    impl<T> Flavor for WriteFlavor<T>
    where
        T: embedded_io::Write,
    {
        type Output = T;

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> Result<()> {
            self.writer
                .write_all(&[data])
                .map_err(|_| Error::SerializeBufferFull)?;
            Ok(())
        }

        #[inline(always)]
        fn try_extend(&mut self, b: &[u8]) -> Result<()> {
            self.writer
                .write_all(b)
                .map_err(|_| Error::SerializeBufferFull)?;
            Ok(())
        }

        fn finalize(mut self) -> Result<Self::Output> {
            self.writer
                .flush()
                .map_err(|_| Error::SerializeBufferFull)?;
            Ok(self.writer)
        }
    }
}

pub mod de {
    use core::marker::PhantomData;
    use postcard2::{Error, Result};

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
        fn take_n(&mut self, ct: usize) -> Result<&'de mut [u8]> {
            let remain = (self.end as usize) - (self.cursor as usize);
            let buff = if remain < ct {
                return Err(Error::DeserializeUnexpectedEnd);
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
        fn take_n_temp(&mut self, ct: usize) -> Result<&mut [u8]> {
            let remain = (self.end as usize) - (self.cursor as usize);
            let buff = if remain < ct {
                return Err(Error::DeserializeUnexpectedEnd);
            } else {
                unsafe { core::slice::from_raw_parts_mut(self.cursor, ct) }
            };

            Ok(buff)
        }

        fn complete(self) -> Result<&'de mut [u8]> {
            let remain = (self.end as usize) - (self.cursor as usize);
            // SAFETY: `self.cursor` is valid for `remain` elements
            unsafe { Ok(core::slice::from_raw_parts_mut(self.cursor, remain)) }
        }
    }

    // Support for [`embedded_io`] traits
    use postcard2::de_flavors::Flavor;

    /// Wrapper over a [`embedded_io`](embedded_io::embedded_io)::[`Read`](embedded_io::Read) and a sliding buffer to implement the [`Flavor`] trait
    pub struct EIOReader<'de, T>
    where
        T: embedded_io::Read,
    {
        reader: T,
        buff: SlidingBuffer<'de>,
    }

    impl<'de, T> EIOReader<'de, T>
    where
        T: embedded_io::Read,
    {
        /// Create a new [`EIOReader`] from a reader and a buffer.
        ///
        /// `buff` must have enough space to hold all data read during the deserialisation.
        pub fn new(reader: T, buff: &'de mut [u8]) -> Self {
            Self {
                reader,
                buff: SlidingBuffer::new(buff),
            }
        }
    }

    impl<'de, T> Flavor<'de> for EIOReader<'de, T>
    where
        T: embedded_io::Read + 'de,
    {
        type Remainder = (T, &'de mut [u8]);
        type Source = &'de [u8];

        #[inline]
        fn pop(&mut self) -> Result<u8> {
            let mut val = [0; 1];
            self.reader
                .read_exact(&mut val)
                .map_err(|_| Error::DeserializeUnexpectedEnd)?;
            Ok(val[0])
        }

        #[inline]
        fn size_hint(&self) -> Option<usize> {
            None
        }

        #[inline]
        fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8]> {
            let buff = self.buff.take_n(ct)?;
            self.reader
                .read_exact(buff)
                .map_err(|_| Error::DeserializeUnexpectedEnd)?;
            Ok(buff)
        }

        #[inline]
        fn try_take_n_temp<'a>(&'a mut self, ct: usize) -> Result<&'a [u8]>
        where
            'de: 'a,
        {
            let buff = self.buff.take_n_temp(ct)?;
            self.reader
                .read_exact(buff)
                .map_err(|_| Error::DeserializeUnexpectedEnd)?;
            Ok(buff)
        }

        /// Return the remaining (unused) bytes in the Deserializer
        fn finalize(self) -> Result<(T, &'de mut [u8])> {
            let buf = self.buff.complete()?;
            Ok((self.reader, buf))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_pop() {
            let mut reader = EIOReader::new(&[0xAA, 0xBB, 0xCC][..], &mut []);

            assert_eq!(reader.pop(), Ok(0xAA));
            assert_eq!(reader.pop(), Ok(0xBB));
            assert_eq!(reader.pop(), Ok(0xCC));
            assert_eq!(reader.pop(), Err(Error::DeserializeUnexpectedEnd));
        }

        #[test]
        fn test_try_take_n() {
            let mut buf = [0; 8];
            let mut reader = EIOReader::new(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE][..], &mut buf);

            assert_eq!(reader.try_take_n(2), Ok(&[0xAA, 0xBB][..]));
            assert_eq!(reader.try_take_n(2), Ok(&[0xCC, 0xDD][..]));
            assert_eq!(reader.try_take_n(2), Err(Error::DeserializeUnexpectedEnd));
        }
    }
}
