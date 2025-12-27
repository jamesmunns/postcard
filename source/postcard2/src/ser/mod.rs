use core::convert::Infallible;

use crate::ser::flavors::{Flavor, Slice};
use flavors::BufferFull;
use serde_core::Serialize;
use serializer::SerializerError;

#[cfg(any(feature = "alloc", feature = "std"))]
use crate::ser::flavors::AllocVec;

#[cfg(any(feature = "alloc", feature = "std"))]
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
pub fn to_slice<'a, 'b, T>(
    value: &'b T,
    buf: &'a mut [u8],
) -> Result<&'a mut [u8], SerializerError<BufferFull, Infallible>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, Slice<'a>>(value, Slice::new(buf))
}

/// Serialize a `T` to an `alloc::vec::Vec<u8>`.
///
/// ## Example
///
/// ```rust
/// use postcard2::to_vec;
///
/// let ser: Vec<u8> = to_vec(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x01]);
///
/// let ser: Vec<u8> = to_vec("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x03, b'H', b'i', b'!']);
/// ```
#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn to_vec<T>(value: &T) -> Result<alloc::vec::Vec<u8>, SerializerError<Infallible, Infallible>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, AllocVec>(value, AllocVec::new())
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
pub fn to_extend<T, W>(value: &T, writer: W) -> Result<W, SerializerError<Infallible, Infallible>>
where
    T: Serialize + ?Sized,
    W: core::iter::Extend<u8>,
{
    serialize_with_flavor::<T, _>(value, flavors::ExtendFlavor::new(writer))
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
pub fn to_io<T, W>(
    value: &T,
    writer: W,
) -> Result<W, SerializerError<std::io::Error, std::io::Error>>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    serialize_with_flavor::<T, _>(value, flavors::io::WriteFlavor::new(writer))
}

/// `serialize_with_flavor()` has three generic parameters, `T, F, O`.
///
/// * `T`: This is the type that is being serialized
/// * `S`: This is the Storage that is used during serialization
///
/// For more information about how Flavors work, please see the
/// [`flavors` module documentation](./flavors/index.html).
pub fn serialize_with_flavor<T, S>(
    value: &T,
    storage: S,
) -> Result<S::Output, SerializerError<S::PushError, S::FinalizeError>>
where
    T: Serialize + ?Sized,
    S: Flavor,
{
    let mut serializer = Serializer { output: storage };
    value.serialize(&mut serializer)?;
    serializer
        .output
        .finalize()
        .map_err(SerializerError::FinalizeError)
}

/// Compute the size of the postcard serialization of `T`.
pub fn serialized_size<T>(value: &T) -> Result<usize, SerializerError<Infallible, Infallible>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, flavors::Size>(value, flavors::Size::default())
}
