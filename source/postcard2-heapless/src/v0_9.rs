use core::convert::Infallible;
use postcard2::{BufferFull, SerializerError, serialize_with_flavor};
use serde_core::Serialize;

/// Serialize a `T` to a `heapless_v0_9::Vec<u8>`, with the `Vec` containing
/// data in a serialized format.
///
/// ## Example
///
/// ```rust
/// use postcard2_heapless::v0_9::to_vec;
/// use heapless_v0_9::Vec;
/// use core::ops::Deref;
///
/// let ser: Vec<u8, 32> = to_vec(&true).unwrap();
/// assert_eq!(ser.deref(), &[0x01]);
///
/// let ser: Vec<u8, 32> = to_vec("Hi!").unwrap();
/// assert_eq!(ser.deref(), &[0x03, b'H', b'i', b'!']);
///
/// // NOTE: postcard handles `&[u8]` and `&[u8; N]` differently.
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, 32> = to_vec(data).unwrap();
/// assert_eq!(ser.deref(), &[0x04, 0x01, 0x00, 0x20, 0x30]);
///
/// let data: &[u8; 4] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, 32> = to_vec(data).unwrap();
/// assert_eq!(ser.deref(), &[0x01, 0x00, 0x20, 0x30]);
/// ```
pub fn to_vec<T, const B: usize>(
    value: &T,
) -> Result<heapless_v0_9::Vec<u8, B>, SerializerError<BufferFull, Infallible>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, ser::HVec<B>>(value, ser::HVec::default())
}

pub mod ser {
    use core::convert::Infallible;
    use core::ops::Index;
    use core::ops::IndexMut;
    use heapless_v0_9::Vec;
    use postcard2::{BufferFull, ser_flavors::Flavor};

    ////////////////////////////////////////
    // HVec
    ////////////////////////////////////////

    /// The `HVec` flavor is a wrapper type around a `heapless_v0_9::Vec`. This is a stack
    /// allocated data structure, with a fixed maximum size and variable amount of contents.
    #[derive(Default)]
    pub struct HVec<const B: usize> {
        /// the contained data buffer
        vec: Vec<u8, B>,
    }

    impl<const B: usize> HVec<B> {
        /// Create a new, currently empty, [`heapless_v0_9::Vec`] to be used for storing serialized
        /// output data.
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl<const B: usize> Flavor for HVec<B> {
        type Output = Vec<u8, B>;
        type PushError = BufferFull;
        type FinalizeError = Infallible;

        #[inline(always)]
        fn try_extend(&mut self, data: &[u8]) -> Result<(), BufferFull> {
            self.vec.extend_from_slice(data).map_err(|_| BufferFull)
        }

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> Result<(), BufferFull> {
            self.vec.push(data).map_err(|_| BufferFull)
        }

        fn finalize(self) -> Result<Vec<u8, B>, Infallible> {
            Ok(self.vec)
        }
    }

    impl<const B: usize> Index<usize> for HVec<B> {
        type Output = u8;

        fn index(&self, idx: usize) -> &u8 {
            &self.vec[idx]
        }
    }

    impl<const B: usize> IndexMut<usize> for HVec<B> {
        fn index_mut(&mut self, idx: usize) -> &mut u8 {
            &mut self.vec[idx]
        }
    }
}

#[cfg(test)]
mod test_heapless {
    use super::*;
    // use crate::{ser::to_vec, to_vec_cobs, varint::varint_max};
    use core::fmt::Write;
    use core::ops::Deref;
    use heapless_v0_9::{String, Vec, index_map::FnvIndexMap};
    use postcard2::{from_bytes, varint::varint_max};
    use serde_core::{Deserialize, Deserializer, Serialize, Serializer};

    #[test]
    fn de_u8() {
        let output: Vec<u8, 1> = to_vec(&0x05u8).unwrap();
        assert_eq!(&[5], output.deref());

        let out: u8 = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 0x05);
    }

    #[test]
    fn de_u16() {
        let output: Vec<u8, { varint_max::<u16>() }> = to_vec(&0xA5C7u16).unwrap();
        assert_eq!(&[0xC7, 0xCB, 0x02], output.deref());

        let out: u16 = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 0xA5C7);
    }

    #[test]
    fn de_u32() {
        let output: Vec<u8, { varint_max::<u32>() }> = to_vec(&0xCDAB3412u32).unwrap();
        assert_eq!(&[0x92, 0xE8, 0xAC, 0xED, 0x0C], output.deref());

        let out: u32 = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 0xCDAB3412u32);
    }

    #[test]
    fn de_u64() {
        let output: Vec<u8, { varint_max::<u64>() }> = to_vec(&0x1234_5678_90AB_CDEFu64).unwrap();
        assert_eq!(
            &[0xEF, 0x9B, 0xAF, 0x85, 0x89, 0xCF, 0x95, 0x9A, 0x12],
            output.deref()
        );

        let out: u64 = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 0x1234_5678_90AB_CDEFu64);
    }

    #[test]
    fn de_u128() {
        let output: Vec<u8, { varint_max::<u128>() }> =
            to_vec(&0x1234_5678_90AB_CDEF_1234_5678_90AB_CDEFu128).unwrap();
        assert_eq!(
            &[
                0xEF, 0x9B, 0xAF, 0x85, 0x89, 0xCF, 0x95, 0x9A, 0x92, 0xDE, 0xB7, 0xDE, 0x8A, 0x92,
                0x9E, 0xAB, 0xB4, 0x24,
            ],
            output.deref()
        );

        let out: u128 = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 0x1234_5678_90AB_CDEF_1234_5678_90AB_CDEFu128);
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
    struct BasicU8S {
        st: u16,
        ei: u8,
        ote: u128,
        sf: u64,
        tt: u32,
    }

    #[test]
    fn de_struct_unsigned() {
        let data = BasicU8S {
            st: 0xABCD,
            ei: 0xFE,
            ote: 0x1234_4321_ABCD_DCBA_1234_4321_ABCD_DCBA,
            sf: 0x1234_4321_ABCD_DCBA,
            tt: 0xACAC_ACAC,
        };

        const SZ: usize = varint_max::<u16>()
            + 1
            + varint_max::<u128>()
            + varint_max::<u64>()
            + varint_max::<u32>();

        let output: Vec<u8, SZ> = to_vec(&data).unwrap();

        assert_eq!(
            &[
                0xCD, 0xD7, 0x02, 0xFE, 0xBA, 0xB9, 0xB7, 0xDE, 0x9A, 0xE4, 0x90, 0x9A, 0x92, 0xF4,
                0xF2, 0xEE, 0xBC, 0xB5, 0xC8, 0xA1, 0xB4, 0x24, 0xBA, 0xB9, 0xB7, 0xDE, 0x9A, 0xE4,
                0x90, 0x9A, 0x12, 0xAC, 0xD9, 0xB2, 0xE5, 0x0A
            ],
            output.deref()
        );

        let out: BasicU8S = from_bytes(output.deref()).unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn de_byte_slice() {
        let input: &[u8] = &[1u8, 2, 3, 4, 5, 6, 7, 8];
        let output: Vec<u8, 9> = to_vec(input).unwrap();
        assert_eq!(
            &[0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            output.deref()
        );

        let out: Vec<u8, 128> = from_bytes(output.deref()).unwrap();
        assert_eq!(input, out.deref());

        let mut input: Vec<u8, 1024> = Vec::new();
        for i in 0..1024 {
            input.push((i & 0xFF) as u8).unwrap();
        }
        let output: Vec<u8, 2048> = to_vec(input.deref()).unwrap();
        assert_eq!(&[0x80, 0x08], &output.deref()[..2]);

        assert_eq!(output.len(), 1026);
        for (i, val) in output.deref()[2..].iter().enumerate() {
            assert_eq!((i & 0xFF) as u8, *val);
        }

        let de: Vec<u8, 1024> = from_bytes(output.deref()).unwrap();
        assert_eq!(input.deref(), de.deref());
    }

    #[test]
    fn de_str() {
        let input: &str = "hello, postcard!";
        let output: Vec<u8, 17> = to_vec(input).unwrap();
        assert_eq!(0x10, output.deref()[0]);
        assert_eq!(input.as_bytes(), &output.deref()[1..]);

        let mut input: String<1024> = String::new();
        for _ in 0..256 {
            write!(&mut input, "abcd").unwrap();
        }
        let output: Vec<u8, 2048> = to_vec(input.deref()).unwrap();
        assert_eq!(&[0x80, 0x08], &output.deref()[..2]);

        assert_eq!(output.len(), 1026);
        for ch in output.deref()[2..].chunks(4) {
            assert_eq!("abcd", core::str::from_utf8(ch).unwrap());
        }

        let de: String<1024> = from_bytes(output.deref()).unwrap();
        assert_eq!(input.deref(), de.deref());
    }

    #[allow(dead_code)]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
    enum BasicEnum {
        Bib,
        Bim,
        Bap,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
    struct EnumStruct {
        eight: u8,
        sixt: u16,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
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
        let output: Vec<u8, 1> = to_vec(&BasicEnum::Bim).unwrap();
        assert_eq!(&[0x01], output.deref());
        let out: BasicEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, BasicEnum::Bim);

        let output: Vec<u8, { 1 + varint_max::<u64>() }> =
            to_vec(&DataEnum::Bim(u64::MAX)).unwrap();
        assert_eq!(
            &[
                0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01
            ],
            output.deref()
        );

        let output: Vec<u8, { 1 + varint_max::<u16>() }> =
            to_vec(&DataEnum::Bib(u16::MAX)).unwrap();
        assert_eq!(&[0x00, 0xFF, 0xFF, 0x03], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Bib(u16::MAX));

        let output: Vec<u8, 2> = to_vec(&DataEnum::Bap(u8::MAX)).unwrap();
        assert_eq!(&[0x02, 0xFF], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Bap(u8::MAX));

        let output: Vec<u8, 8> = to_vec(&DataEnum::Kim(EnumStruct {
            eight: 0xF0,
            sixt: 0xACAC,
        }))
        .unwrap();
        assert_eq!(&[0x03, 0xF0, 0xAC, 0xD9, 0x02], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(
            out,
            DataEnum::Kim(EnumStruct {
                eight: 0xF0,
                sixt: 0xACAC
            })
        );

        let output: Vec<u8, 8> = to_vec(&DataEnum::Chi {
            a: 0x0F,
            b: 0xC7C7C7C7,
        })
        .unwrap();
        assert_eq!(&[0x04, 0x0F, 0xC7, 0x8F, 0x9F, 0xBE, 0x0C], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(
            out,
            DataEnum::Chi {
                a: 0x0F,
                b: 0xC7C7C7C7
            }
        );

        let output: Vec<u8, 8> = to_vec(&DataEnum::Sho(0x6969, 0x07)).unwrap();
        assert_eq!(&[0x05, 0xE9, 0xD2, 0x01, 0x07], output.deref());
        let out: DataEnum = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DataEnum::Sho(0x6969, 0x07));
    }

    #[test]
    fn tuples() {
        let output: Vec<u8, 128> = to_vec(&(1u8, 10u32, "Hello!")).unwrap();
        assert_eq!(
            &[1u8, 0x0A, 0x06, b'H', b'e', b'l', b'l', b'o', b'!'],
            output.deref()
        );
        let out: (u8, u32, &str) = from_bytes(output.deref()).unwrap();
        assert_eq!(out, (1u8, 10u32, "Hello!"));
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct ByteSliceStruct<'a> {
        bytes: &'a [u8],
    }

    impl Serialize for ByteSliceStruct<'_> {
        fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // Serialization is generic for all slice types, so the default serialization of byte
            // slices does not use `Serializer::serialize_bytes`.
            serializer.serialize_bytes(self.bytes)
        }
    }

    impl<'a, 'de> Deserialize<'de> for ByteSliceStruct<'a>
    where
        'de: 'a,
    {
        fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Deserialization of byte slices is specialized for byte slices, so the default
            // deserialization will call `Deserializer::deserialize_bytes`.
            Ok(Self {
                bytes: Deserialize::deserialize(deserializer)?,
            })
        }
    }

    #[test]
    fn bytes() {
        let x: &[u8; 32] = &[0u8; 32];
        let output: Vec<u8, 128> = to_vec(x).unwrap();
        assert_eq!(output.len(), 32);
        let out: [u8; 32] = from_bytes(output.deref()).unwrap();
        assert_eq!(out, [0u8; 32]);

        let x: &[u8] = &[0u8; 32];
        let output: Vec<u8, 128> = to_vec(x).unwrap();
        assert_eq!(output.len(), 33);
        let out: &[u8] = from_bytes(output.deref()).unwrap();
        assert_eq!(out, [0u8; 32]);

        let x = ByteSliceStruct { bytes: &[0u8; 32] };
        let output: Vec<u8, 128> = to_vec(&x).unwrap();
        assert_eq!(output.len(), 33);
        let out: ByteSliceStruct<'_> = from_bytes(output.deref()).unwrap();
        assert_eq!(out, ByteSliceStruct { bytes: &[0u8; 32] });
    }

    #[test]
    fn chars() {
        let x: char = 'a';
        let output: Vec<u8, 5> = to_vec(&x).unwrap();
        assert_eq!(output.len(), 2);
        let out: char = from_bytes(output.deref()).unwrap();
        assert_eq!(out, 'a');
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
    pub struct NewTypeStruct(u32);

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
    pub struct TupleStruct((u8, u16));

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
    pub struct DualTupleStruct(u8, u16);

    #[test]
    fn structs() {
        let output: Vec<u8, 4> = to_vec(&NewTypeStruct(5)).unwrap();
        assert_eq!(&[0x05], output.deref());
        let out: NewTypeStruct = from_bytes(output.deref()).unwrap();
        assert_eq!(out, NewTypeStruct(5));

        let output: Vec<u8, 3> = to_vec(&TupleStruct((0xA0, 0x1234))).unwrap();
        assert_eq!(&[0xA0, 0xB4, 0x24], output.deref());
        let out: TupleStruct = from_bytes(output.deref()).unwrap();
        assert_eq!(out, TupleStruct((0xA0, 0x1234)));

        let output: Vec<u8, 3> = to_vec(&DualTupleStruct(0xA0, 0x1234)).unwrap();
        assert_eq!(&[0xA0, 0xB4, 0x24], output.deref());
        let out: DualTupleStruct = from_bytes(output.deref()).unwrap();
        assert_eq!(out, DualTupleStruct(0xA0, 0x1234));
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
    struct RefStruct<'a> {
        bytes: &'a [u8],
        str_s: &'a str,
    }

    #[test]
    fn ref_struct() {
        let message = "hElLo";
        let bytes = [0x01, 0x10, 0x02, 0x20];
        let output: Vec<u8, 11> = to_vec(&RefStruct {
            bytes: &bytes,
            str_s: message,
        })
        .unwrap();

        assert_eq!(
            &[
                0x04, 0x01, 0x10, 0x02, 0x20, 0x05, b'h', b'E', b'l', b'L', b'o',
            ],
            output.deref()
        );

        let out: RefStruct<'_> = from_bytes(output.deref()).unwrap();
        assert_eq!(
            out,
            RefStruct {
                bytes: &bytes,
                str_s: message,
            }
        );
    }

    #[test]
    fn unit() {
        let output: Vec<u8, 1> = to_vec(&()).unwrap();
        assert_eq!(output.len(), 0);
        let _: () = from_bytes(output.deref()).unwrap();
    }

    #[test]
    fn heapless_data() {
        let mut input: Vec<u8, 4> = Vec::new();
        input.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let output: Vec<u8, 5> = to_vec(&input).unwrap();
        assert_eq!(&[0x04, 0x01, 0x02, 0x03, 0x04], output.deref());
        let out: Vec<u8, 4> = from_bytes(output.deref()).unwrap();
        assert_eq!(out, input);

        let mut input: String<8> = String::new();
        write!(&mut input, "helLO!").unwrap();
        let output: Vec<u8, 7> = to_vec(&input).unwrap();
        assert_eq!(&[0x06, b'h', b'e', b'l', b'L', b'O', b'!'], output.deref());
        let out: String<8> = from_bytes(output.deref()).unwrap();
        assert_eq!(input, out);

        let mut input: FnvIndexMap<u8, u8, 4> = FnvIndexMap::new();
        input.insert(0x01, 0x05).unwrap();
        input.insert(0x02, 0x06).unwrap();
        input.insert(0x03, 0x07).unwrap();
        input.insert(0x04, 0x08).unwrap();
        let output: Vec<u8, 100> = to_vec(&input).unwrap();
        assert_eq!(
            &[0x04, 0x01, 0x05, 0x02, 0x06, 0x03, 0x07, 0x04, 0x08],
            output.deref()
        );
        let out: FnvIndexMap<u8, u8, 4> = from_bytes(output.deref()).unwrap();
        assert_eq!(input, out);
    }

    // #[test]
    // fn cobs_test() {
    //     let message = "hElLo";
    //     let bytes = [0x01, 0x00, 0x02, 0x20];
    //     let input = RefStruct {
    //         bytes: &bytes,
    //         str_s: message,
    //     };

    //     let output: Vec<u8, 11> = to_vec(&input).unwrap();

    //     let mut encode_buf = [0u8; 32];
    //     let sz = cobs::encode(output.deref(), &mut encode_buf);
    //     let out = from_bytes_cobs::<RefStruct<'_>>(&mut encode_buf[..sz]).unwrap();

    //     assert_eq!(input, out);
    // }

    // #[test]
    // fn take_from_includes_terminator() {
    //     // With the null terminator
    //     let mut output: Vec<u8, 32> = to_vec_cobs(&(4i32, 0u8, 4u64)).unwrap();
    //     let (val, remain) = take_from_bytes_cobs::<(i32, u8, u64)>(&mut output).unwrap();
    //     assert_eq!((4, 0, 4), val);
    //     assert_eq!(remain.len(), 0);

    //     // without the null terminator
    //     let mut output: Vec<u8, 32> = to_vec_cobs(&(4i32, 0u8, 4u64)).unwrap();
    //     assert_eq!(output.pop(), Some(0));
    //     let (val, remain) = take_from_bytes_cobs::<(i32, u8, u64)>(&mut output).unwrap();
    //     assert_eq!((4, 0, 4), val);
    //     assert_eq!(remain.len(), 0);
    // }
}

// #[cfg(test)]
// mod test_ser {
//     use super::*;
//     use crate::max_size::MaxSize;
//     use crate::varint::{varint_max, varint_usize};
//     use core::fmt::Write;
//     use core::ops::{Deref, DerefMut};
//     use heapless_v0_9::{FnvIndexMap, String};
//     use serde_core::Deserialize;

//     #[test]
//     fn ser_u8() {
//         let output: Vec<u8, 1> = to_vec(&0x05u8).unwrap();
//         assert_eq!(&[5], output.deref());
//         assert!(output.len() == serialized_size(&0x05u8).unwrap());
//         assert!(output.len() <= Vec::<u8, 1>::POSTCARD_MAX_SIZE);
//     }

//     #[test]
//     fn ser_u16() {
//         const SZ: usize = varint_max::<u16>();
//         let output: Vec<u8, SZ> = to_vec(&0xA5C7u16).unwrap();
//         assert_eq!(&[0xC7, 0xCB, 0x02], output.deref());
//         assert!(output.len() == serialized_size(&0xA5C7u16).unwrap());
//         assert!(output.len() <= Vec::<u8, SZ>::POSTCARD_MAX_SIZE);
//     }

//     #[test]
//     fn ser_u32() {
//         const SZ: usize = varint_max::<u32>();
//         let output: Vec<u8, SZ> = to_vec(&0xCDAB3412u32).unwrap();
//         assert_eq!(&[0x92, 0xE8, 0xAC, 0xED, 0x0C], output.deref());
//         assert!(output.len() == serialized_size(&0xCDAB3412u32).unwrap());
//         assert!(output.len() <= Vec::<u8, SZ>::POSTCARD_MAX_SIZE);
//     }

//     #[test]
//     fn ser_u64() {
//         const SZ: usize = varint_max::<u64>();
//         let output: Vec<u8, SZ> = to_vec(&0x1234_5678_90AB_CDEFu64).unwrap();
//         assert_eq!(
//             &[0xEF, 0x9B, 0xAF, 0x85, 0x89, 0xCF, 0x95, 0x9A, 0x12],
//             output.deref()
//         );
//         assert!(output.len() == serialized_size(&0x1234_5678_90AB_CDEFu64).unwrap());
//         assert!(output.len() <= Vec::<u8, SZ>::POSTCARD_MAX_SIZE);
//     }

//     #[test]
//     fn ser_u128() {
//         const SZ: usize = varint_max::<u128>();
//         let output: Vec<u8, SZ> = to_vec(&0x1234_5678_90AB_CDEF_1234_5678_90AB_CDEFu128).unwrap();
//         assert_eq!(
//             &[
//                 0xEF, 0x9B, 0xAF, 0x85, 0x89, 0xCF, 0x95, 0x9A, 0x92, 0xDE, 0xB7, 0xDE, 0x8A, 0x92,
//                 0x9E, 0xAB, 0xB4, 0x24,
//             ],
//             output.deref()
//         );
//         assert!(
//             output.len()
//                 == serialized_size(&0x1234_5678_90AB_CDEF_1234_5678_90AB_CDEFu128).unwrap()
//         );
//         assert!(output.len() <= Vec::<u8, SZ>::POSTCARD_MAX_SIZE);
//     }

//     #[derive(Serialize)]
//     struct BasicU8S {
//         st: u16,
//         ei: u8,
//         ote: u128,
//         sf: u64,
//         tt: u32,
//     }

//     impl MaxSize for BasicU8S {
//         const POSTCARD_MAX_SIZE: usize = {
//             u16::POSTCARD_MAX_SIZE
//                 + u8::POSTCARD_MAX_SIZE
//                 + u128::POSTCARD_MAX_SIZE
//                 + u64::POSTCARD_MAX_SIZE
//                 + u32::POSTCARD_MAX_SIZE
//         };
//     }

//     #[test]
//     fn ser_struct_unsigned() {
//         const SZ: usize = BasicU8S::POSTCARD_MAX_SIZE;
//         let input = BasicU8S {
//             st: 0xABCD,
//             ei: 0xFE,
//             ote: 0x1234_4321_ABCD_DCBA_1234_4321_ABCD_DCBA,
//             sf: 0x1234_4321_ABCD_DCBA,
//             tt: 0xACAC_ACAC,
//         };
//         let output: Vec<u8, SZ> = to_vec(&input).unwrap();

//         assert_eq!(
//             &[
//                 0xCD, 0xD7, 0x02, 0xFE, 0xBA, 0xB9, 0xB7, 0xDE, 0x9A, 0xE4, 0x90, 0x9A, 0x92, 0xF4,
//                 0xF2, 0xEE, 0xBC, 0xB5, 0xC8, 0xA1, 0xB4, 0x24, 0xBA, 0xB9, 0xB7, 0xDE, 0x9A, 0xE4,
//                 0x90, 0x9A, 0x12, 0xAC, 0xD9, 0xB2, 0xE5, 0x0A
//             ],
//             output.deref()
//         );
//         assert!(output.len() == serialized_size(&input).unwrap());
//         assert!(output.len() <= BasicU8S::POSTCARD_MAX_SIZE);
//     }

//     #[test]
//     fn ser_byte_slice() {
//         let input: &[u8] = &[1u8, 2, 3, 4, 5, 6, 7, 8];
//         let output: Vec<u8, 9> = to_vec(input).unwrap();
//         assert_eq!(
//             &[0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
//             output.deref()
//         );
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let mut input: Vec<u8, 1024> = Vec::new();
//         for i in 0..1024 {
//             input.push((i & 0xFF) as u8).unwrap();
//         }
//         let output: Vec<u8, 2048> = to_vec(input.deref()).unwrap();
//         assert_eq!(&[0x80, 0x08], &output.deref()[..2]);

//         assert_eq!(output.len(), 1026);
//         for (i, val) in output.deref()[2..].iter().enumerate() {
//             assert_eq!((i & 0xFF) as u8, *val);
//         }
//     }

//     #[test]
//     fn ser_str() {
//         let input: &str = "hello, postcard!";
//         let output: Vec<u8, 17> = to_vec(input).unwrap();
//         assert_eq!(0x10, output.deref()[0]);
//         assert_eq!(input.as_bytes(), &output.deref()[1..]);
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let mut input: String<1024> = String::new();
//         for _ in 0..256 {
//             write!(&mut input, "abcd").unwrap();
//         }
//         let output: Vec<u8, 2048> = to_vec(input.deref()).unwrap();
//         assert_eq!(&[0x80, 0x08], &output.deref()[..2]);
//         assert!(String::<1024>::POSTCARD_MAX_SIZE <= output.len());

//         assert_eq!(output.len(), 1026);
//         for ch in output.deref()[2..].chunks(4) {
//             assert_eq!("abcd", core::str::from_utf8(ch).unwrap());
//         }
//     }

//     #[test]
//     fn usize_varint_encode() {
//         let mut buf = [0; varint_max::<usize>()];
//         let res = varint_usize(1, &mut buf);

//         assert_eq!(&[1], res);

//         let res = varint_usize(usize::MAX, &mut buf);

//         //
//         if varint_max::<usize>() == varint_max::<u32>() {
//             assert_eq!(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F], res);
//         } else if varint_max::<usize>() == varint_max::<u64>() {
//             assert_eq!(
//                 &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
//                 res
//             );
//         } else {
//             panic!("Update this test for 16/128 bit targets!");
//         }
//     }

//     #[allow(dead_code)]
//     #[derive(Serialize)]
//     enum BasicEnum {
//         Bib,
//         Bim,
//         Bap,
//     }

//     #[derive(Serialize)]
//     struct EnumStruct {
//         eight: u8,
//         sixt: u16,
//     }

//     #[derive(Serialize)]
//     enum DataEnum {
//         Bib(u16),
//         Bim(u64),
//         Bap(u8),
//         Kim(EnumStruct),
//         Chi { a: u8, b: u32 },
//         Sho(u16, u8),
//     }

//     #[test]
//     fn enums() {
//         let input = BasicEnum::Bim;
//         let output: Vec<u8, 1> = to_vec(&input).unwrap();
//         assert_eq!(&[0x01], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let input = DataEnum::Bim(u64::MAX);
//         let output: Vec<u8, { 1 + varint_max::<u64>() }> = to_vec(&input).unwrap();
//         assert_eq!(
//             &[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
//             output.deref()
//         );
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let input = DataEnum::Bib(u16::MAX);
//         let output: Vec<u8, { 1 + varint_max::<u16>() }> = to_vec(&input).unwrap();
//         assert_eq!(&[0x00, 0xFF, 0xFF, 0x03], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let input = DataEnum::Bap(u8::MAX);
//         let output: Vec<u8, 2> = to_vec(&input).unwrap();
//         assert_eq!(&[0x02, 0xFF], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let input = DataEnum::Kim(EnumStruct {
//             eight: 0xF0,
//             sixt: 0xACAC,
//         });
//         let output: Vec<u8, 8> = to_vec(&input).unwrap();
//         assert_eq!(&[0x03, 0xF0, 0xAC, 0xD9, 0x02], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let input = DataEnum::Chi {
//             a: 0x0F,
//             b: 0xC7C7C7C7,
//         };
//         let output: Vec<u8, 8> = to_vec(&input).unwrap();
//         assert_eq!(&[0x04, 0x0F, 0xC7, 0x8F, 0x9F, 0xBE, 0x0C], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let input = DataEnum::Sho(0x6969, 0x07);
//         let output: Vec<u8, 8> = to_vec(&input).unwrap();
//         assert_eq!(&[0x05, 0xE9, 0xD2, 0x01, 0x07], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());
//     }

//     #[test]
//     fn tuples() {
//         let input = (1u8, 10u32, "Hello!");
//         let output: Vec<u8, 128> = to_vec(&input).unwrap();
//         assert_eq!(
//             &[1u8, 0x0A, 0x06, b'H', b'e', b'l', b'l', b'o', b'!'],
//             output.deref()
//         );
//         assert!(output.len() == serialized_size(&input).unwrap());
//     }

//     #[test]
//     fn bytes() {
//         let x: &[u8; 32] = &[0u8; 32];
//         let output: Vec<u8, 128> = to_vec(x).unwrap();
//         assert_eq!(output.len(), 32);
//         assert!(output.len() == serialized_size(&x).unwrap());
//         assert!(<[u8; 32] as MaxSize>::POSTCARD_MAX_SIZE <= output.len());

//         let x: &[u8] = &[0u8; 32];
//         let output: Vec<u8, 128> = to_vec(x).unwrap();
//         assert_eq!(output.len(), 33);
//         assert!(output.len() == serialized_size(&x).unwrap());
//     }

//     #[derive(Serialize)]
//     pub struct NewTypeStruct(u32);

//     #[derive(Serialize)]
//     pub struct TupleStruct((u8, u16));

//     #[test]
//     fn structs() {
//         let input = NewTypeStruct(5);
//         let output: Vec<u8, 1> = to_vec(&input).unwrap();
//         assert_eq!(&[0x05], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let input = TupleStruct((0xA0, 0x1234));
//         let output: Vec<u8, 3> = to_vec(&input).unwrap();
//         assert_eq!(&[0xA0, 0xB4, 0x24], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());
//     }

//     #[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug)]
//     struct RefStruct<'a> {
//         bytes: &'a [u8],
//         str_s: &'a str,
//     }

//     #[test]
//     fn ref_struct() {
//         let message = "hElLo";
//         let bytes = [0x01, 0x10, 0x02, 0x20];
//         let input = RefStruct {
//             bytes: &bytes,
//             str_s: message,
//         };
//         let output: Vec<u8, 11> = to_vec(&input).unwrap();

//         assert_eq!(
//             &[0x04, 0x01, 0x10, 0x02, 0x20, 0x05, b'h', b'E', b'l', b'L', b'o',],
//             output.deref()
//         );
//         assert!(output.len() == serialized_size(&input).unwrap());
//     }

//     #[test]
//     fn unit() {
//         let output: Vec<u8, 1> = to_vec(&()).unwrap();
//         assert_eq!(output.len(), 0);
//         assert!(output.len() == serialized_size(&()).unwrap());
//     }

//     #[test]
//     fn heapless_data() {
//         let mut input: Vec<u8, 4> = Vec::new();
//         input.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
//         let output: Vec<u8, 5> = to_vec(&input).unwrap();
//         assert_eq!(&[0x04, 0x01, 0x02, 0x03, 0x04], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let mut input: String<8> = String::new();
//         write!(&mut input, "helLO!").unwrap();
//         let output: Vec<u8, 7> = to_vec(&input).unwrap();
//         assert_eq!(&[0x06, b'h', b'e', b'l', b'L', b'O', b'!'], output.deref());
//         assert!(output.len() == serialized_size(&input).unwrap());

//         let mut input: FnvIndexMap<u8, u8, 4> = FnvIndexMap::new();
//         input.insert(0x01, 0x05).unwrap();
//         input.insert(0x02, 0x06).unwrap();
//         input.insert(0x03, 0x07).unwrap();
//         input.insert(0x04, 0x08).unwrap();
//         let output: Vec<u8, 100> = to_vec(&input).unwrap();
//         assert_eq!(
//             &[0x04, 0x01, 0x05, 0x02, 0x06, 0x03, 0x07, 0x04, 0x08],
//             output.deref()
//         );
//         assert!(output.len() == serialized_size(&input).unwrap());
//     }

//     // #[test]
//     // fn cobs_test() {
//     //     let message = "hElLo";
//     //     let bytes = [0x01, 0x00, 0x02, 0x20];
//     //     let input = RefStruct {
//     //         bytes: &bytes,
//     //         str_s: message,
//     //     };

//     //     let mut output: Vec<u8, 13> = to_vec_cobs(&input).unwrap();

//     //     let sz = cobs::decode_in_place(output.deref_mut()).unwrap();

//     //     let x = crate::from_bytes::<RefStruct<'_>>(&output.deref_mut()[..sz]).unwrap();

//     //     assert_eq!(input, x);
//     // }

//     #[test]
//     fn test_vec_edge_cases() {
//         #[track_caller]
//         fn test_equals<const N: usize>(buf: &mut [u8]) {
//             let mut v = heapless_v0_9::Vec::<u8, N>::new();
//             for _ in 0..N {
//                 v.push(0).unwrap();
//             }

//             let serialized = postcard2::to_slice(&v, buf).unwrap();

//             assert_eq!(heapless_v0_9::Vec::<u8, N>::POSTCARD_MAX_SIZE, serialized.len());
//         }

//         let mut buf = [0; 16400];

//         test_equals::<1>(&mut buf);
//         test_equals::<2>(&mut buf);

//         test_equals::<127>(&mut buf);
//         test_equals::<128>(&mut buf);
//         test_equals::<129>(&mut buf);

//         test_equals::<16383>(&mut buf);
//         test_equals::<16384>(&mut buf);
//         test_equals::<16385>(&mut buf);
//     }
// }
