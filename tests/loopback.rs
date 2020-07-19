#![allow(unused_imports)]

use core::fmt::Debug;
use core::fmt::Write;
use core::ops::Deref;

#[cfg(feature = "heapless")]
use heapless::{consts::*, String, Vec, FnvIndexMap};

#[cfg(feature = "heapless")]
use postcard::to_vec;

use postcard::from_bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct BasicU8S {
    st: u16,
    ei: u8,
    sf: u64,
    tt: u32,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
enum BasicEnum {
    Bib,
    Bim,
    Bap,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct EnumStruct {
    eight: u8,
    sixt: u16,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
enum DataEnum {
    Bib(u16),
    Bim(u64),
    Bap(u8),
    Kim(EnumStruct),
    Chi { a: u8, b: u32 },
    Sho(u16, u8),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct NewTypeStruct(u32);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct TupleStruct((u8, u16));

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct RefStruct<'a> {
    bytes: &'a [u8],
    str_s: &'a str,
}

#[cfg(feature = "heapless")]
#[test]
fn loopback() {
    // Basic types
    test_one((), &[]);
    test_one(false, &[0x00]);
    test_one(true, &[0x01]);
    test_one(5u8, &[0x05]);
    test_one(0xA5C7u16, &[0xC7, 0xA5]);
    test_one(0xCDAB3412u32, &[0x12, 0x34, 0xAB, 0xCD]);
    test_one(
        0x1234_5678_90AB_CDEFu64,
        &[0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12],
    );

    // Structs
    test_one(
        BasicU8S {
            st: 0xABCD,
            ei: 0xFE,
            sf: 0x1234_4321_ABCD_DCBA,
            tt: 0xACAC_ACAC,
        },
        &[
            0xCD, 0xAB, 0xFE, 0xBA, 0xDC, 0xCD, 0xAB, 0x21, 0x43, 0x34, 0x12, 0xAC, 0xAC, 0xAC,
            0xAC,
        ],
    );

    // Slices
    // TODO(AJM) - why aren't slices impl Deserialize?
    // let sl_1: &[u8; 8] = &[0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    // test_one(
    //     sl_1,
    //     &[0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
    // );

    // AJM(TODO)
    // let mut input: Vec<u8, U1024> = Vec::new();
    // let mut output: ::std::vec::Vec<u8> = vec![];
    // output.push(0x80);
    // output.push(0x08); //  0x0800
    // for i in 0..1024 {
    //     input.push((i & 0xFF) as u8).unwrap();
    //     output.push((i & 0xFF) as u8);
    // }
    // let x: &[u8] = input.deref();
    // test_one(x, &output);

    // TODO(AJM) - port de_str test
    // test_one(
    //     "hello, postcard!",
    //     &[0x10, b'h', b'e', b'l', b'l', b'o', b',', b' ', b'p', b'o', b's', b't', b'c', b'a', b'r', b'd', b'!']
    // );

    // Enums!
    test_one(BasicEnum::Bim, &[0x01]);
    test_one(
        DataEnum::Bim(u64::max_value()),
        &[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    );
    test_one(DataEnum::Bib(u16::max_value()), &[0x00, 0xFF, 0xFF]);
    test_one(DataEnum::Bap(u8::max_value()), &[0x02, 0xFF]);
    test_one(
        DataEnum::Kim(EnumStruct {
            eight: 0xF0,
            sixt: 0xACAC,
        }),
        &[0x03, 0xF0, 0xAC, 0xAC],
    );
    test_one(
        DataEnum::Chi {
            a: 0x0F,
            b: 0xC7C7C7C7,
        },
        &[0x04, 0x0F, 0xC7, 0xC7, 0xC7, 0xC7],
    );
    test_one(DataEnum::Sho(0x6969, 0x07), &[0x05, 0x69, 0x69, 0x07]);

    // Tuples!
    test_one((0x12u8, 0xC7A5u16), &[0x12, 0xA5, 0xC7]);

    // TODO(AJM)
    // test_one(
    //     (1u8, 10u32, "Hello!"),
    //     &[1u8, 0x0A, 0x00, 0x00, 0x00, 0x06, b'H', b'e', b'l', b'l', b'o', b'!']
    // );

    // Structs!
    test_one(NewTypeStruct(5), &[0x05, 0x00, 0x00, 0x00]);
    test_one(TupleStruct((0xA0, 0x1234)), &[0xA0, 0x34, 0x12]);

    // Ref Struct
    // let message = "hElLo";
    // let bytes = [0x01, 0x10, 0x02, 0x20];
    // test_one(
    //     RefStruct {
    //         bytes: &bytes,
    //         str_s: message,
    //     },
    //     &[0x04, 0x01, 0x10, 0x02, 0x20, 0x05, b'h', b'E', b'l', b'L', b'o',]
    // );

    let mut input: Vec<u8, U4> = Vec::new();
    input.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
    test_one(input, &[0x04, 0x01, 0x02, 0x03, 0x04]);

    let mut input: String<U8> = String::new();
    write!(&mut input, "helLO!").unwrap();
    test_one(input, &[0x06, b'h', b'e', b'l', b'L', b'O', b'!']);

    let mut input: FnvIndexMap<u8, u8, U4> = FnvIndexMap::new();
    input.insert(0x01, 0x05).unwrap();
    input.insert(0x02, 0x06).unwrap();
    input.insert(0x03, 0x07).unwrap();
    input.insert(0x04, 0x08).unwrap();
    test_one(input, &[0x04, 0x01, 0x05, 0x02, 0x06, 0x03, 0x07, 0x04, 0x08]);

    // `CString` (uses `serialize_bytes`/`deserialize_byte_buf`)
    #[cfg(feature = "use-std")]
    test_one(std::ffi::CString::new("heLlo").unwrap(), &[0x05, b'h', b'e', b'L', b'l', b'o']);
}

#[cfg(feature = "heapless")]
fn test_one<'a, 'de, T>(data: T, ser_rep: &'a [u8])
where
    T: Serialize + DeserializeOwned + Eq + PartialEq + Debug,
{
    let serialized: Vec<u8, U2048> = to_vec(&data).unwrap();
    assert_eq!(serialized.len(), ser_rep.len());
    let mut x: ::std::vec::Vec<u8> = vec![];
    x.extend(serialized.deref().iter().cloned());
    // let bysl: &'de [u8] = serialized.deref();
    assert_eq!(x, ser_rep);
    {
        // let deserialized: T = from_bytes(serialized.deref()).unwrap();
        let deserialized: T = from_bytes(&x).unwrap();
        assert_eq!(data, deserialized);
    }
}
