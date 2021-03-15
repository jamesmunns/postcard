use serde::Serialize;
use crate::error::{Error, Result};
use crate::ser::flavors::{Cobs, SerFlavor, Slice};

#[cfg(feature = "heapless")]
use crate::ser::flavors::HVec;

#[cfg(feature = "heapless")]
use heapless::{ArrayLength, Vec};

#[cfg(feature = "use-std")]
use crate::ser::flavors::StdVec;

#[cfg(feature = "alloc")]
use crate::ser::flavors::AllocVec;

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::ser::serializer::Serializer;

pub mod flavors;
pub(crate) mod serializer;

/// Serialize a `T` to the given slice, with the resulting slice containing
/// data in a serialized then COBS encoded format. The terminating sentinel `0x00` byte is included
/// in the output buffer.
///
/// When successful, this function returns the slices containing:
///
/// 1. A slice that contains the serialized and encoded message
/// 2. A slice that contains the unused portion of the given buffer
///
/// ## Example
///
/// ```rust
/// use postcard::to_slice_cobs;
/// let mut buf = [0u8; 32];
///
/// let used = to_slice_cobs(&false, &mut buf).unwrap();
/// assert_eq!(used, &[0x01, 0x01, 0x00]);
///
/// let used = to_slice_cobs("1", &mut buf).unwrap();
/// assert_eq!(used, &[0x03, 0x01, b'1', 0x00]);
///
/// let used = to_slice_cobs("Hi!", &mut buf).unwrap();
/// assert_eq!(used, &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
///
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let used = to_slice_cobs(data, &mut buf).unwrap();
/// assert_eq!(used, &[0x03, 0x04, 0x01, 0x03, 0x20, 0x30, 0x00]);
/// ```
pub fn to_slice_cobs<'a, 'b, T>(value: &'b T, buf: &'a mut [u8]) -> Result<&'a mut [u8]>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, Cobs<Slice<'a>>, &'a mut [u8]>(
        value,
        Cobs::try_new(Slice::new(buf))?,
    )
}

/// Serialize a `T` to the given slice, with the resulting slice containing
/// data in a serialized format.
///
/// When successful, this function returns the slices containing:
///
/// 1. A slice that contains the serialized message
/// 2. A slice that contains the unused portion of the given buffer
///
/// ## Example
///
/// ```rust
/// use postcard::to_slice;
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

/// Serialize a `T` to a `heapless::Vec<u8>`, with the `Vec` containing
/// data in a serialized then COBS encoded format. The terminating sentinel
/// `0x00` byte is included in the output `Vec`. Requires the (default) `heapless` feature.
///
/// ## Example
///
/// ```rust
/// use postcard::to_vec_cobs;
/// use heapless::{Vec, consts::*};
/// use core::ops::Deref;
///
/// let ser: Vec<u8, U32> = to_vec_cobs(&false).unwrap();
/// assert_eq!(ser.deref(), &[0x01, 0x01, 0x00]);
///
/// let ser: Vec<u8, U32> = to_vec_cobs("Hi!").unwrap();
/// assert_eq!(ser.deref(), &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
///
/// // NOTE: postcard handles `&[u8]` and `&[u8; N]` differently.
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, U32> = to_vec_cobs(data).unwrap();
/// assert_eq!(ser.deref(), &[0x03, 0x04, 0x01, 0x03, 0x20, 0x30, 0x00]);
///
/// let data: &[u8; 4] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, U32> = to_vec_cobs(data).unwrap();
/// assert_eq!(ser.deref(), &[0x02, 0x01, 0x03, 0x20, 0x30, 0x00]);
/// ```
#[cfg(feature = "heapless")]
pub fn to_vec_cobs<B, T>(value: &T) -> Result<Vec<u8, B>>
where
    T: Serialize + ?Sized,
    B: ArrayLength<u8>,
{
    serialize_with_flavor::<T, Cobs<HVec<_>>, Vec<u8, B>>(value, Cobs::try_new(HVec::default())?)
}

/// Serialize a `T` to a `heapless::Vec<u8>`, with the `Vec` containing
/// data in a serialized format. Requires the (default) `heapless` feature.
///
/// ## Example
///
/// ```rust
/// use postcard::to_vec;
/// use heapless::{Vec, consts::*};
/// use core::ops::Deref;
///
/// let ser: Vec<u8, U32> = to_vec(&true).unwrap();
/// assert_eq!(ser.deref(), &[0x01]);
///
/// let ser: Vec<u8, U32> = to_vec("Hi!").unwrap();
/// assert_eq!(ser.deref(), &[0x03, b'H', b'i', b'!']);
///
/// // NOTE: postcard handles `&[u8]` and `&[u8; N]` differently.
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, U32> = to_vec(data).unwrap();
/// assert_eq!(ser.deref(), &[0x04, 0x01, 0x00, 0x20, 0x30]);
///
/// let data: &[u8; 4] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, U32> = to_vec(data).unwrap();
/// assert_eq!(ser.deref(), &[0x01, 0x00, 0x20, 0x30]);
/// ```
#[cfg(feature = "heapless")]
pub fn to_vec<B, T>(value: &T) -> Result<Vec<u8, B>>
where
    T: Serialize + ?Sized,
    B: ArrayLength<u8>,
{
    serialize_with_flavor::<T, HVec<B>, Vec<u8, B>>(value, HVec::default())
}

/// Serialize a `T` to a `std::vec::Vec<u8>`. Requires the `use-std` feature.
///
/// ## Example
///
/// ```rust
/// use postcard::to_stdvec;
///
/// let ser: Vec<u8> = to_stdvec(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x01]);
///
/// let ser: Vec<u8> = to_stdvec("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "use-std")]
pub fn to_stdvec<T>(value: &T) -> Result<std::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, StdVec, std::vec::Vec<u8>>(value, StdVec(std::vec::Vec::new()))
}

/// Serialize and COBS encode a `T` to a `std::vec::Vec<u8>`. Requires the `use-std` feature.
///
/// The terminating sentinel `0x00` byte is included in the output.
///
/// ## Example
///
/// ```rust
/// use postcard::to_stdvec_cobs;
///
/// let ser: Vec<u8> = to_stdvec_cobs(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x02, 0x01, 0x00]);
///
/// let ser: Vec<u8> = to_stdvec_cobs("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
/// ```
#[cfg(feature = "use-std")]
pub fn to_stdvec_cobs<T>(value: &T) -> Result<std::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, Cobs<StdVec>, std::vec::Vec<u8>>(
        value,
        Cobs::try_new(StdVec(std::vec::Vec::new()))?,
    )
}

/// Serialize a `T` to an `alloc::vec::Vec<u8>`. Requires the `alloc` feature.
///
/// ## Example
///
/// ```rust
/// use postcard::to_allocvec;
///
/// let ser: Vec<u8> = to_allocvec(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x01]);
///
/// let ser: Vec<u8> = to_allocvec("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "alloc")]
pub fn to_allocvec<T>(value: &T) -> Result<alloc::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, AllocVec, alloc::vec::Vec<u8>>(value, AllocVec(alloc::vec::Vec::new()))
}

/// Serialize and COBS encode a `T` to an `alloc::vec::Vec<u8>`. Requires the `alloc` feature.
///
/// The terminating sentinel `0x00` byte is included in the output.
///
/// ## Example
///
/// ```rust
/// use postcard::to_allocvec_cobs;
///
/// let ser: Vec<u8> = to_allocvec_cobs(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x02, 0x01, 0x00]);
///
/// let ser: Vec<u8> = to_allocvec_cobs("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
/// ```
#[cfg(feature = "alloc")]
pub fn to_allocvec_cobs<T>(value: &T) -> Result<alloc::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, Cobs<AllocVec>, alloc::vec::Vec<u8>>(
        value,
        Cobs::try_new(AllocVec(alloc::vec::Vec::new()))?,
    )
}

/// `serialize_with_flavor()` has three generic parameters, `T, F, O`.
///
/// * `T`: This is the type that is being serialized
/// * `F`: This is the Flavor (combination) that is used during serialization
/// * `O`: This is the resulting storage type that is returned containing the serialized data
///
/// For more information about how Flavors work, please see the
/// [`flavors` module documentation](./flavors/index.html).
///
/// ```rust
/// use postcard::{
///     serialize_with_flavor,
///     flavors::{Cobs, Slice},
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
pub fn serialize_with_flavor<T, F, O>(value: &T, flavor: F) -> Result<O>
where
    T: Serialize + ?Sized,
    F: SerFlavor<Output = O>,
{
    let mut serializer = Serializer { output: flavor };
    value.serialize(&mut serializer)?;
    serializer
        .output
        .release()
        .map_err(|_| Error::SerializeBufferFull)
}

#[cfg(feature = "heapless")]
#[cfg(test)]
mod test {
    use super::*;
    use crate::varint::VarintUsize;
    use core::fmt::Write;
    use core::ops::{Deref, DerefMut};
    use heapless::{consts::*, String, FnvIndexMap};
    use serde::Deserialize;

    #[test]
    fn ser_u8() {
        let output: Vec<u8, U1> = to_vec(&0x05u8).unwrap();
        assert!(&[5] == output.deref());
    }

    #[test]
    fn ser_u16() {
        let output: Vec<u8, U2> = to_vec(&0xA5C7u16).unwrap();
        assert!(&[0xC7, 0xA5] == output.deref());
    }

    #[test]
    fn ser_u32() {
        let output: Vec<u8, U4> = to_vec(&0xCDAB3412u32).unwrap();
        assert!(&[0x12, 0x34, 0xAB, 0xCD] == output.deref());
    }

    #[test]
    fn ser_u64() {
        let output: Vec<u8, U8> = to_vec(&0x1234_5678_90AB_CDEFu64).unwrap();
        assert!(&[0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12] == output.deref());
    }

    #[test]
    fn ser_u128() {
        let output: Vec<u8, U16> = to_vec(&0x1234_5678_90AB_CDEF_1234_5678_90AB_CDEFu128).unwrap();
        assert!(
            &[
                0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12,
                0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12
            ] == output.deref()
        );
    }

    #[derive(Serialize)]
    struct BasicU8S {
        st: u16,
        ei: u8,
        ote: u128,
        sf: u64,
        tt: u32,
    }

    #[test]
    fn ser_struct_unsigned() {
        let output: Vec<u8, U31> = to_vec(&BasicU8S {
            st: 0xABCD,
            ei: 0xFE,
            ote: 0x1234_4321_ABCD_DCBA_1234_4321_ABCD_DCBA,
            sf: 0x1234_4321_ABCD_DCBA,
            tt: 0xACAC_ACAC,
        })
        .unwrap();

        assert!(
            &[
                0xCD, 0xAB,
                0xFE,
                0xBA, 0xDC, 0xCD, 0xAB, 0x21, 0x43, 0x34, 0x12,
                0xBA, 0xDC, 0xCD, 0xAB, 0x21, 0x43, 0x34, 0x12,
                0xBA, 0xDC, 0xCD, 0xAB, 0x21, 0x43, 0x34, 0x12,
                0xAC, 0xAC, 0xAC, 0xAC
            ] == output.deref()
        );
    }

    #[test]
    fn ser_byte_slice() {
        let input: &[u8] = &[1u8, 2, 3, 4, 5, 6, 7, 8];
        let output: Vec<u8, U9> = to_vec(input).unwrap();
        assert_eq!(
            &[0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            output.deref()
        );

        let mut input: Vec<u8, U1024> = Vec::new();
        for i in 0..1024 {
            input.push((i & 0xFF) as u8).unwrap();
        }
        let output: Vec<u8, U2048> = to_vec(input.deref()).unwrap();
        assert_eq!(&[0x80, 0x08], &output.deref()[..2]);

        assert_eq!(output.len(), 1026);
        for (i, val) in output.deref()[2..].iter().enumerate() {
            assert_eq!((i & 0xFF) as u8, *val);
        }
    }

    #[test]
    fn ser_str() {
        let input: &str = "hello, postcard!";
        let output: Vec<u8, U17> = to_vec(input).unwrap();
        assert_eq!(0x10, output.deref()[0]);
        assert_eq!(input.as_bytes(), &output.deref()[1..]);

        let mut input: String<U1024> = String::new();
        for _ in 0..256 {
            write!(&mut input, "abcd").unwrap();
        }
        let output: Vec<u8, U2048> = to_vec(input.deref()).unwrap();
        assert_eq!(&[0x80, 0x08], &output.deref()[..2]);

        assert_eq!(output.len(), 1026);
        for ch in output.deref()[2..].chunks(4) {
            assert_eq!("abcd", core::str::from_utf8(ch).unwrap());
        }
    }

    #[test]
    fn usize_varint_encode() {
        let mut buf = VarintUsize::new_buf();
        let res = VarintUsize(1).to_buf(&mut buf);

        assert!(&[1] == res);

        let res = VarintUsize(usize::max_value()).to_buf(&mut buf);

        // AJM TODO
        if VarintUsize::varint_usize_max() == 5 {
            assert_eq!(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F], res);
        } else {
            assert_eq!(
                &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
                res
            );
        }
    }

    #[allow(dead_code)]
    #[derive(Serialize)]
    enum BasicEnum {
        Bib,
        Bim,
        Bap,
    }

    #[derive(Serialize)]
    struct EnumStruct {
        eight: u8,
        sixt: u16,
    }

    #[derive(Serialize)]
    enum DataEnum {
        Bib(u16),
        Bim(u64),
        Bap(u8),
        Kim(EnumStruct),
        Chi { a: u8, b: u32 },
        Sho(u16, u8),
    }

    #[test]
    fn enums() {
        let output: Vec<u8, U1> = to_vec(&BasicEnum::Bim).unwrap();
        assert_eq!(&[0x01], output.deref());

        let output: Vec<u8, U9> = to_vec(&DataEnum::Bim(u64::max_value())).unwrap();
        assert_eq!(
            &[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            output.deref()
        );

        let output: Vec<u8, U3> = to_vec(&DataEnum::Bib(u16::max_value())).unwrap();
        assert_eq!(&[0x00, 0xFF, 0xFF], output.deref());

        let output: Vec<u8, U2> = to_vec(&DataEnum::Bap(u8::max_value())).unwrap();
        assert_eq!(&[0x02, 0xFF], output.deref());

        let output: Vec<u8, U8> = to_vec(&DataEnum::Kim(EnumStruct {
            eight: 0xF0,
            sixt: 0xACAC,
        }))
        .unwrap();
        assert_eq!(&[0x03, 0xF0, 0xAC, 0xAC,], output.deref());

        let output: Vec<u8, U8> = to_vec(&DataEnum::Chi {
            a: 0x0F,
            b: 0xC7C7C7C7,
        })
        .unwrap();
        assert_eq!(&[0x04, 0x0F, 0xC7, 0xC7, 0xC7, 0xC7], output.deref());

        let output: Vec<u8, U8> = to_vec(&DataEnum::Sho(0x6969, 0x07)).unwrap();
        assert_eq!(&[0x05, 0x69, 0x69, 0x07], output.deref());
    }

    #[test]
    fn tuples() {
        let output: Vec<u8, U128> = to_vec(&(1u8, 10u32, "Hello!")).unwrap();
        assert_eq!(
            &[1u8, 0x0A, 0x00, 0x00, 0x00, 0x06, b'H', b'e', b'l', b'l', b'o', b'!'],
            output.deref()
        )
    }

    #[test]
    fn bytes() {
        let x: &[u8; 32] = &[0u8; 32];
        let output: Vec<u8, U128> = to_vec(x).unwrap();
        assert_eq!(output.len(), 32);

        let x: &[u8] = &[0u8; 32];
        let output: Vec<u8, U128> = to_vec(x).unwrap();
        assert_eq!(output.len(), 33);
    }

    #[derive(Serialize)]
    pub struct NewTypeStruct(u32);

    #[derive(Serialize)]
    pub struct TupleStruct((u8, u16));

    #[test]
    fn structs() {
        let output: Vec<u8, U4> = to_vec(&NewTypeStruct(5)).unwrap();
        assert_eq!(&[0x05, 0x00, 0x00, 0x00], output.deref());

        let output: Vec<u8, U3> = to_vec(&TupleStruct((0xA0, 0x1234))).unwrap();
        assert_eq!(&[0xA0, 0x34, 0x12], output.deref());
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    struct RefStruct<'a> {
        bytes: &'a [u8],
        str_s: &'a str,
    }

    #[test]
    fn ref_struct() {
        let message = "hElLo";
        let bytes = [0x01, 0x10, 0x02, 0x20];
        let output: Vec<u8, U11> = to_vec(&RefStruct {
            bytes: &bytes,
            str_s: message,
        })
        .unwrap();

        assert_eq!(
            &[0x04, 0x01, 0x10, 0x02, 0x20, 0x05, b'h', b'E', b'l', b'L', b'o',],
            output.deref()
        );
    }

    #[test]
    fn unit() {
        let output: Vec<u8, U1> = to_vec(&()).unwrap();
        assert_eq!(output.len(), 0);
    }

    #[test]
    fn heapless_data() {
        let mut input: Vec<u8, U4> = Vec::new();
        input.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let output: Vec<u8, U5> = to_vec(&input).unwrap();
        assert_eq!(&[0x04, 0x01, 0x02, 0x03, 0x04], output.deref());

        let mut input: String<U8> = String::new();
        write!(&mut input, "helLO!").unwrap();
        let output: Vec<u8, U7> = to_vec(&input).unwrap();
        assert_eq!(&[0x06, b'h', b'e', b'l', b'L', b'O', b'!'], output.deref());

        let mut input: FnvIndexMap<u8, u8, U4> = FnvIndexMap::new();
        input.insert(0x01, 0x05).unwrap();
        input.insert(0x02, 0x06).unwrap();
        input.insert(0x03, 0x07).unwrap();
        input.insert(0x04, 0x08).unwrap();
        let output: Vec<u8, U100> = to_vec(&input).unwrap();
        assert_eq!(&[0x04, 0x01, 0x05, 0x02, 0x06, 0x03, 0x07, 0x04, 0x08], output.deref());
    }

    #[test]
    fn cobs_test() {
        let message = "hElLo";
        let bytes = [0x01, 0x00, 0x02, 0x20];
        let input = RefStruct {
            bytes: &bytes,
            str_s: message,
        };

        let mut output: Vec<u8, U13> = to_vec_cobs(&input).unwrap();

        let sz = cobs::decode_in_place(output.deref_mut()).unwrap();

        let x = crate::from_bytes::<RefStruct>(&output.deref_mut()[..sz]).unwrap();

        assert_eq!(input, x);
    }
}
