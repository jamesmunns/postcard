use crate::error::{Error, Result};
use crate::ser::flavors::{Flavor, Slice};
use serde_core::Serialize;

#[cfg(feature = "alloc")]
use crate::ser::flavors::AllocVec;

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::ser::serializer::Serializer;

pub mod flavors;
pub(crate) mod serializer;

/// Serialize a `T` to the given slice, with the resulting slice containing
/// data in a serialized format.
///
/// When successful, this function returns the slice containing the
/// serialized message
///
/// ## Example
///
/// ```rust
/// use postcard2::to_slice;
/// let mut buf = [0u8; 32];
///
/// let used = to_slice(&true, &mut buf).unwrap();
/// assert_eq!(used, &[0x01]);
///
/// let used = to_slice("Hi!", &mut buf).unwrap();
/// assert_eq!(used, &[0x03, b'H', b'i', b'!']);
///
/// // NOTE: postcard handles `&[u8]` and `&[u8; N]` differently.
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let used = to_slice(data, &mut buf).unwrap();
/// assert_eq!(used, &[0x04, 0x01, 0x00, 0x20, 0x30]);
///
/// let data: &[u8; 4] = &[0x01u8, 0x00, 0x20, 0x30];
/// let used = to_slice(data, &mut buf).unwrap();
/// assert_eq!(used, &[0x01, 0x00, 0x20, 0x30]);
/// ```
pub fn to_slice<'a, 'b, T>(value: &'b T, buf: &'a mut [u8]) -> Result<&'a mut [u8]>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, Slice<'a>, &'a mut [u8]>(value, Slice::new(buf))
}

/// Serialize a `T` to a `std::vec::Vec<u8>`.
///
/// ## Example
///
/// ```rust
/// use postcard2::to_stdvec;
///
/// let ser: Vec<u8> = to_stdvec(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x01]);
///
/// let ser: Vec<u8> = to_stdvec("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[inline]
pub fn to_stdvec<T>(value: &T) -> Result<std::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    to_allocvec(value)
}

/// Serialize a `T` to an `alloc::vec::Vec<u8>`.
///
/// ## Example
///
/// ```rust
/// use postcard2::to_allocvec;
///
/// let ser: Vec<u8> = to_allocvec(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x01]);
///
/// let ser: Vec<u8> = to_allocvec("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn to_allocvec<T>(value: &T) -> Result<alloc::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, AllocVec, alloc::vec::Vec<u8>>(value, AllocVec::new())
}

/// Serialize a `T` to a [`core::iter::Extend`],
/// ## Example
///
/// ```rust
/// use postcard2::to_extend;
/// let mut vec = Vec::new();
///
/// let ser = to_extend(&true, vec).unwrap();
/// let vec = to_extend("Hi!", ser).unwrap();
/// assert_eq!(&vec[0..5], &[0x01, 0x03, b'H', b'i', b'!']);
/// ```
pub fn to_extend<T, W>(value: &T, writer: W) -> Result<W>
where
    T: Serialize + ?Sized,
    W: core::iter::Extend<u8>,
{
    serialize_with_flavor::<T, _, _>(value, flavors::ExtendFlavor::new(writer))
}

/// Serialize a `T` to a [`std::io::Write`],
/// ## Example
///
/// ```rust
/// use postcard2::to_io;
/// let mut buf: [u8; 32] = [0; 32];
/// let mut writer: &mut [u8] = &mut buf;
///
/// let ser = to_io(&true, &mut writer).unwrap();
/// to_io("Hi!", ser).unwrap();
/// assert_eq!(&buf[0..5], &[0x01, 0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "std")]
pub fn to_io<T, W>(value: &T, writer: W) -> Result<W>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    serialize_with_flavor::<T, _, _>(value, flavors::io::WriteFlavor::new(writer))
}

/// `serialize_with_flavor()` has three generic parameters, `T, F, O`.
///
/// * `T`: This is the type that is being serialized
/// * `S`: This is the Storage that is used during serialization
/// * `O`: This is the resulting storage type that is returned containing the serialized data
///
/// For more information about how Flavors work, please see the
/// [`flavors` module documentation](./flavors/index.html).
///
/// ```rust
/// use postcard2::{
///     serialize_with_flavor,
///     ser_flavors::{Cobs, Slice},
/// };
///
/// let mut buf = [0u8; 32];
///
/// let data: &[u8] = &[0x01, 0x00, 0x20, 0x30];
/// let buffer = &mut [0u8; 32];
/// let res = serialize_with_flavor::<[u8], Cobs<Slice>, &mut [u8]>(
///     data,
///     Cobs::try_new(Slice::new(buffer)).unwrap(),
/// ).unwrap();
///
/// assert_eq!(res, &[0x03, 0x04, 0x01, 0x03, 0x20, 0x30, 0x00]);
/// ```
pub fn serialize_with_flavor<T, S, O>(value: &T, storage: S) -> Result<O>
where
    T: Serialize + ?Sized,
    S: Flavor<Output = O>,
{
    let mut serializer = Serializer { output: storage };
    value.serialize(&mut serializer)?;
    serializer
        .output
        .finalize()
        .map_err(|_| Error::SerializeBufferFull)
}

/// Compute the size of the postcard serialization of `T`.
pub fn serialized_size<T>(value: &T) -> Result<usize>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, flavors::Size, usize>(value, flavors::Size::default())
}
