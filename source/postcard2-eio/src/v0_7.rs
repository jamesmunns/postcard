//! Support Utilities for embedded-io v0.7.x

use postcard2::{Deserializer, DeserializerError, SerializerError, serialize_with_flavor};
use serde_core::{Deserialize, Serialize};

/// Serialize a `T` to an [`embedded_io Write`](embedded_io_v0_7::Write),
/// ## Example
///
/// ```rust
/// use postcard2_eio::v0_7::to_eio;
/// let mut buf: [u8; 32] = [0; 32];
/// let mut writer: &mut [u8] = &mut buf;
///
/// let ser = to_eio(&true, &mut writer).unwrap();
/// to_eio("Hi!", ser).unwrap();
/// assert_eq!(&buf[0..5], &[0x01, 0x03, b'H', b'i', b'!']);
/// ```
pub fn to_eio<T, W>(value: &T, writer: W) -> Result<W, SerializerError<W::Error, W::Error>>
where
    T: Serialize + ?Sized,
    W: embedded_io_v0_7::Write,
{
    serialize_with_flavor::<T, _>(value, ser::WriteFlavor::new(writer))
}

pub type EioDeserRemainder<'a, R> = (R, &'a mut [u8]);
pub type EioDeserError<R> = DeserializerError<de::IoError<R>, de::IoError<R>>;

/// Deserialize a message of type `T` from a [`embedded_io`](embedded_io_v0_7)::[`Read`](embedded_io_v0_7::Read).
pub fn from_eio<'a, T, R>(
    val: (R, &'a mut [u8]),
) -> Result<(T, EioDeserRemainder<'a, R>), EioDeserError<R::Error>>
where
    T: Deserialize<'a>,
    R: embedded_io_v0_7::Read + 'a,
{
    let flavor = de::EIOReader::new(val.0, val.1);
    let mut deserializer = Deserializer::from_flavor(flavor);
    let t = T::deserialize(&mut deserializer)?;
    Ok((t, deserializer.finalize()?))
}

pub mod ser {
    use postcard2::ser_flavors::Flavor;

    /// Wrapper over a [`embedded_io Write`](embedded_io_v0_7::Write) that implements the flavor trait
    pub struct WriteFlavor<T> {
        writer: T,
    }

    impl<T> WriteFlavor<T>
    where
        T: embedded_io_v0_7::Write,
    {
        /// Create a new [`Self`] flavor from a given [`embedded_io Write`](embedded_io_v0_7::Write)
        pub fn new(writer: T) -> Self {
            Self { writer }
        }
    }

    impl<T> Flavor for WriteFlavor<T>
    where
        T: embedded_io_v0_7::Write,
    {
        type Output = T;
        type PushError = T::Error;
        type FinalizeError = T::Error;

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> Result<(), Self::PushError> {
            self.writer.write_all(&[data])?;
            Ok(())
        }

        #[inline(always)]
        fn try_extend(&mut self, b: &[u8]) -> Result<(), Self::PushError> {
            self.writer.write_all(b)?;
            Ok(())
        }

        fn finalize(mut self) -> Result<Self::Output, Self::FinalizeError> {
            self.writer.flush()?;
            Ok(self.writer)
        }
    }
}

pub mod de {
    use core::marker::PhantomData;
    use postcard2::de_flavors::Flavor;

    #[derive(Debug, PartialEq, Eq)]
    pub enum IoError<ReadErr> {
        BufferExhausted,
        ReadError(embedded_io_v0_7::ReadExactError<ReadErr>),
    }

    impl<ReadErr> core::fmt::Display for IoError<ReadErr>
    where
        ReadErr: core::fmt::Display + core::fmt::Debug,
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::BufferExhausted => f.write_str("BufferExhausted"),
                Self::ReadError(e) => write!(f, "ReadError({e})"),
            }
        }
    }

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
        fn take_n<ReadErr>(&mut self, ct: usize) -> Result<&'de mut [u8], IoError<ReadErr>> {
            let remain = (self.end as usize) - (self.cursor as usize);
            let buff = if remain < ct {
                return Err(IoError::BufferExhausted);
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
        fn take_n_temp<ReadErr>(&mut self, ct: usize) -> Result<&mut [u8], IoError<ReadErr>> {
            let remain = (self.end as usize) - (self.cursor as usize);
            let buff = if remain < ct {
                return Err(IoError::BufferExhausted);
            } else {
                unsafe { core::slice::from_raw_parts_mut(self.cursor, ct) }
            };

            Ok(buff)
        }

        fn complete(self) -> &'de mut [u8] {
            let remain = (self.end as usize) - (self.cursor as usize);
            // SAFETY: `self.cursor` is valid for `remain` elements
            unsafe { core::slice::from_raw_parts_mut(self.cursor, remain) }
        }
    }

    /// Wrapper over a [`embedded_io`](embedded_io_v0_7)::[`Read`](embedded_io_v0_7::Read) and a sliding buffer to implement the [`Flavor`] trait
    pub struct EIOReader<'de, T>
    where
        T: embedded_io_v0_7::Read,
        T::Error: core::fmt::Display + core::fmt::Debug,
    {
        reader: T,
        buff: SlidingBuffer<'de>,
    }

    impl<'de, T> EIOReader<'de, T>
    where
        T: embedded_io_v0_7::Read,
        T::Error: core::fmt::Display + core::fmt::Debug,
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
        T: embedded_io_v0_7::Read + 'de,
        T::Error: core::fmt::Display + core::fmt::Debug,
    {
        type Remainder = (T, &'de mut [u8]);
        type Source = &'de [u8];
        type PopError = IoError<T::Error>;
        type FinalizeError = IoError<T::Error>;

        #[inline]
        fn pop(&mut self) -> Result<u8, Self::PopError> {
            let mut val = [0; 1];
            self.reader
                .read_exact(&mut val)
                .map_err(IoError::ReadError)?;
            Ok(val[0])
        }

        #[inline]
        fn size_hint(&self) -> Option<usize> {
            None
        }

        #[inline]
        fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8], Self::PopError> {
            let buff = self.buff.take_n::<T::Error>(ct)?;
            self.reader.read_exact(buff).map_err(IoError::ReadError)?;
            Ok(buff)
        }

        #[inline]
        fn try_take_n_temp<'a>(&'a mut self, ct: usize) -> Result<&'a [u8], Self::PopError>
        where
            'de: 'a,
        {
            let buff = self.buff.take_n_temp::<T::Error>(ct)?;
            self.reader.read_exact(buff).map_err(IoError::ReadError)?;
            Ok(buff)
        }

        /// Return the remaining (unused) bytes in the Deserializer
        fn finalize(self) -> Result<(T, &'de mut [u8]), Self::FinalizeError> {
            Ok((self.reader, self.buff.complete()))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use embedded_io_v0_7::ReadExactError;

        #[test]
        fn test_pop() {
            let mut reader = EIOReader::new(&[0xAA, 0xBB, 0xCC][..], &mut []);

            assert_eq!(reader.pop(), Ok(0xAA));
            assert_eq!(reader.pop(), Ok(0xBB));
            assert_eq!(reader.pop(), Ok(0xCC));
            assert_eq!(
                reader.pop(),
                Err(IoError::ReadError(ReadExactError::UnexpectedEof))
            );
        }

        #[test]
        fn test_try_take_n() {
            let mut buf = [0; 8];
            let mut reader = EIOReader::new(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE][..], &mut buf);

            assert_eq!(reader.try_take_n(2), Ok(&[0xAA, 0xBB][..]));
            assert_eq!(reader.try_take_n(2), Ok(&[0xCC, 0xDD][..]));
            assert_eq!(
                reader.try_take_n(2),
                Err(IoError::ReadError(ReadExactError::UnexpectedEof))
            );
        }
    }
}

// #[test]
// fn std_eio_loopback() {
//     use postcard2::from_eio;
//     use postcard2::to_eio;

//     fn test_io<T>(data: T, ser_rep: &[u8])
//     where
//         T: Serialize + DeserializeOwned + Eq + PartialEq + Debug,
//     {
//         let serialized: ::std::vec::Vec<u8> = vec![];
//         let ser = to_eio(&data, serialized).unwrap();
//         assert_eq!(ser.len(), ser_rep.len());
//         assert_eq!(ser, ser_rep);
//         {
//             let mut buff = [0; 2048];
//             let x = ser.clone();
//             let deserialized: T = from_eio((x.as_slice(), &mut buff)).unwrap().0;
//             assert_eq!(data, deserialized);
//         }
//     }

//     test_io(DataEnum::Sho(0x6969, 0x07), &[0x05, 0xE9, 0xD2, 0x01, 0x07]);
//     test_io(
//         BasicU8S {
//             st: 0xABCD,
//             ei: 0xFE,
//             sf: 0x1234_4321_ABCD_DCBA,
//             tt: 0xACAC_ACAC,
//         },
//         &[
//             0xCD, 0xD7, 0x02, 0xFE, 0xBA, 0xB9, 0xB7, 0xDE, 0x9A, 0xE4, 0x90, 0x9A, 0x12, 0xAC,
//             0xD9, 0xB2, 0xE5, 0x0A,
//         ],
//     );
// }
