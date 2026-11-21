//! Support for serializing into [`heapless` v0.9](heapless) `Vec`s.
//!
//! These are replacements for [`to_vec`](crate::to_vec), [`to_vec_cobs`](crate::to_vec_cobs),
//! [`to_vec_crc32`](crate::to_vec_crc32), and [`HVec`](crate::ser_flavors::HVec), which use
//! `heapless` v0.7.
//!
//! ```rust
//! use postcard::heapless_v0_9::{heapless::Vec, to_vec};
//!
//! let ser: Vec<u8, 32> = to_vec("Hi!").unwrap();
//! assert_eq!(&ser[..], &[0x03, b'H', b'i', b'!']);
//! ```

use crate::error::Result;
use crate::ser::flavors::Cobs;
use crate::ser::serialize_with_flavor;
use serde::Serialize;

/// Re-export of the `heapless` v0.9 crate used by this module.
pub use heapless_v0_9 as heapless;

/// Serialize a `T` to a `heapless::Vec<u8>`.
///
/// ## Example
///
/// ```rust
/// use postcard::heapless_v0_9::{heapless::Vec, to_vec};
///
/// let ser: Vec<u8, 32> = to_vec(&true).unwrap();
/// assert_eq!(&ser[..], &[0x01]);
///
/// // NOTE: postcard handles `&[u8]` and `&[u8; N]` differently.
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, 32> = to_vec(data).unwrap();
/// assert_eq!(&ser[..], &[0x04, 0x01, 0x00, 0x20, 0x30]);
///
/// let data: &[u8; 4] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, 32> = to_vec(data).unwrap();
/// assert_eq!(&ser[..], &[0x01, 0x00, 0x20, 0x30]);
/// ```
pub fn to_vec<T, const B: usize>(value: &T) -> Result<heapless::Vec<u8, B>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor(value, ser::HVec::<B>::new())
}

/// Serialize and COBS encode a `T` to a `heapless::Vec<u8>`.
///
/// The terminating sentinel `0x00` byte is included in the output.
///
/// ## Example
///
/// ```rust
/// use postcard::heapless_v0_9::{heapless::Vec, to_vec_cobs};
///
/// let ser: Vec<u8, 32> = to_vec_cobs(&false).unwrap();
/// assert_eq!(&ser[..], &[0x01, 0x01, 0x00]);
///
/// let ser: Vec<u8, 32> = to_vec_cobs("Hi!").unwrap();
/// assert_eq!(&ser[..], &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
/// ```
pub fn to_vec_cobs<T, const B: usize>(value: &T) -> Result<heapless::Vec<u8, B>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor(value, Cobs::try_new(ser::HVec::<B>::new())?)
}

/// Serialize a `T` to a `heapless::Vec<u8>`, followed by a 32-bit CRC.
///
/// The CRC bytes are included in the output `Vec`. See the
/// [`ser_flavors::crc`](crate::ser_flavors::crc) module to use other CRC widths, with
/// [`ser::HVec`] as the inner flavor.
///
/// ## Example
///
/// ```rust
/// use crc::{Crc, CRC_32_ISCSI};
/// use postcard::heapless_v0_9::{heapless::Vec, to_vec_crc32};
///
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let crc = Crc::<u32>::new(&CRC_32_ISCSI);
/// let ser: Vec<u8, 32> = to_vec_crc32(data, crc.digest()).unwrap();
/// assert_eq!(&ser[..], &[0x04, 0x01, 0x00, 0x20, 0x30, 0x8E, 0xC8, 0x1A, 0x37]);
/// ```
#[cfg(feature = "use-crc")]
#[cfg_attr(docsrs, doc(cfg(feature = "use-crc")))]
pub fn to_vec_crc32<T, const B: usize>(
    value: &T,
    digest: crc::Digest<'_, u32>,
) -> Result<heapless::Vec<u8, B>>
where
    T: Serialize + ?Sized,
{
    use crate::ser::flavors::crc::CrcModifier;
    serialize_with_flavor(value, CrcModifier::new(ser::HVec::<B>::new(), digest))
}

/// Serialization flavors for `heapless` v0.9.
pub mod ser {
    use crate::ser::flavors::Flavor;
    use crate::{Error, Result};
    use core::ops::{Index, IndexMut};
    use heapless_v0_9::{LenType, Vec};

    /// A flavor that serializes into a `heapless::Vec<u8, B, LenT>`.
    ///
    /// `LenT` is the `Vec`'s length type, and defaults to `usize` like `heapless::Vec`.
    ///
    /// ```rust
    /// use postcard::heapless_v0_9::{heapless::Vec, ser::HVec};
    /// use postcard::serialize_with_flavor;
    ///
    /// let ser: Vec<u8, 32, u8> = serialize_with_flavor(&true, HVec::<32, u8>::new()).unwrap();
    /// assert_eq!(&ser[..], &[0x01]);
    /// ```
    pub struct HVec<const B: usize, LenT: LenType = usize> {
        vec: Vec<u8, B, LenT>,
    }

    impl<const B: usize, LenT: LenType> HVec<B, LenT> {
        /// Create a new, empty `HVec` flavor.
        pub fn new() -> Self {
            Self { vec: Vec::new() }
        }
    }

    impl<const B: usize, LenT: LenType> Default for HVec<B, LenT> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<const B: usize, LenT: LenType> Flavor for HVec<B, LenT> {
        type Output = Vec<u8, B, LenT>;

        #[inline(always)]
        fn try_extend(&mut self, data: &[u8]) -> Result<()> {
            self.vec
                .extend_from_slice(data)
                .map_err(|_| Error::SerializeBufferFull)
        }

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> Result<()> {
            self.vec.push(data).map_err(|_| Error::SerializeBufferFull)
        }

        fn finalize(self) -> Result<Vec<u8, B, LenT>> {
            Ok(self.vec)
        }
    }

    impl<const B: usize, LenT: LenType> Index<usize> for HVec<B, LenT> {
        type Output = u8;

        fn index(&self, idx: usize) -> &u8 {
            &self.vec[idx]
        }
    }

    impl<const B: usize, LenT: LenType> IndexMut<usize> for HVec<B, LenT> {
        fn index_mut(&mut self, idx: usize) -> &mut u8 {
            &mut self.vec[idx]
        }
    }
}

#[cfg(test)]
mod test {
    use super::heapless::{String, Vec};
    use super::ser::HVec;
    use super::*;
    use crate::ser::flavors::Flavor;
    use crate::{from_bytes, from_bytes_cobs, Error};
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Demo {
        a: u32,
        b: u8,
        c: String<8>,
    }

    fn demo() -> Demo {
        Demo {
            a: 0x1234_5678,
            b: 0xAA,
            c: String::try_from("hello").unwrap(),
        }
    }

    #[test]
    fn to_vec_round_trip() {
        let ser: Vec<u8, 32> = to_vec(&demo()).unwrap();
        assert_eq!(
            ser.len(),
            crate::experimental::serialized_size(&demo()).unwrap()
        );
        let de: Demo = from_bytes(&ser).unwrap();
        assert_eq!(de, demo());
    }

    #[test]
    fn to_vec_full() {
        assert_eq!(
            to_vec::<_, 4>(&demo()),
            Err::<Vec<u8, 4>, _>(Error::SerializeBufferFull)
        );
    }

    #[test]
    fn to_vec_cobs_round_trip() {
        let mut ser: Vec<u8, 32> = to_vec_cobs(&demo()).unwrap();
        assert_eq!(ser.last(), Some(&0x00));
        assert!(!ser[..ser.len() - 1].contains(&0x00));
        let de: Demo = from_bytes_cobs(&mut ser).unwrap();
        assert_eq!(de, demo());
    }

    #[test]
    fn to_vec_cobs_full() {
        assert_eq!(
            to_vec_cobs::<_, 0>(&()),
            Err::<Vec<u8, 0>, _>(Error::SerializeBufferFull)
        );
    }

    #[cfg(feature = "use-crc")]
    #[test]
    fn to_vec_crc32_round_trip() {
        use crc::{Crc, CRC_32_ISCSI};
        let crc = Crc::<u32>::new(&CRC_32_ISCSI);
        let ser: Vec<u8, 32> = to_vec_crc32(&demo(), crc.digest()).unwrap();
        let de: Demo = crate::from_bytes_crc32(&ser, crc.digest()).unwrap();
        assert_eq!(de, demo());
    }

    #[test]
    fn hvec_len_type() {
        let mut fl = HVec::<4, u8>::new();
        fl.try_extend(&[1, 2, 3]).unwrap();
        fl.try_push(4).unwrap();
        assert_eq!(fl.try_push(5), Err(Error::SerializeBufferFull));
        let out: Vec<u8, 4, u8> = fl.finalize().unwrap();
        assert_eq!(&out[..], &[1, 2, 3, 4]);
    }

    // The v0.9 output must be byte-for-byte identical to the v0.7 functions.
    #[cfg(feature = "heapless")]
    #[test]
    fn matches_heapless_v0_7() {
        let old: ::heapless::Vec<u8, 32> = crate::to_vec(&demo()).unwrap();
        let new: Vec<u8, 32> = to_vec(&demo()).unwrap();
        assert_eq!(&old[..], &new[..]);

        let old: ::heapless::Vec<u8, 32> = crate::to_vec_cobs(&demo()).unwrap();
        let new: Vec<u8, 32> = to_vec_cobs(&demo()).unwrap();
        assert_eq!(&old[..], &new[..]);
    }
}
