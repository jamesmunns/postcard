use std::marker::PhantomData;

#[cfg(feature = "heapless")]
use heapless::Vec;

use postcard::from_bytes;
#[cfg(feature = "heapless")]
use postcard::to_vec;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct A {
    a: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    b: Option<u8>,
    c: u8,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum E {
    A(A),
    B {
        #[serde(skip_serializing_if = "Option::is_none")]
        b: Option<u64>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct C {
    #[serde(skip)]
    ghost: u8,
    phantom: PhantomData<u16>,
}

#[test]
#[cfg(feature = "heapless")]
fn complete() {
    let a = A {
        a: 0xA,
        b: Some(0xB),
        c: 0xC,
    };

    let bytes: Vec<u8, 42> = to_vec(&a).expect("unable to serialize");

    let restored: A = from_bytes(&bytes).expect("unable to deserialize");

    assert_eq!(a, restored);
}

#[test]
#[cfg(feature = "heapless")]
fn partial_struct() {
    let a = A {
        a: 0xA,
        b: None,
        c: 0xC,
    };

    let bytes: Vec<u8, 42> = to_vec(&a).expect("unable to serialize");

    let restored: A = from_bytes(&bytes).expect("unable to deserialize");

    assert_eq!(a, restored);
}

#[test]
#[cfg(feature = "heapless")]
fn partial_enum_a() {
    let e = E::A(A {
        a: 0xA,
        b: None,
        c: 0xC,
    });

    let bytes: Vec<u8, 42> = to_vec(&e).expect("unable to serialize");

    let restored: E = from_bytes(&bytes).expect("unable to deserialize");

    assert_eq!(e, restored);
}

#[test]
#[cfg(feature = "heapless")]
fn partial_enum_b() {
    let e = E::B { b: None };

    let bytes: Vec<u8, 42> = to_vec(&e).expect("unable to serialize");

    let restored: E = from_bytes(&bytes).expect("unable to deserialize");

    assert_eq!(e, restored);
}

#[test]
#[cfg(feature = "heapless")]
fn empty_struct_unconditional() {
    let c = C::default();

    let bytes: Vec<u8, 42> = to_vec(&c).expect("unable to serialize");

    assert_eq!(0, bytes.len());
    //assert!(false, "{:?}", bytes);

    let restored: C = from_bytes(&bytes).expect("unable to deserialize");

    assert_eq!(c, restored);
}
