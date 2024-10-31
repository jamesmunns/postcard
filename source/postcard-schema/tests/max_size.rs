#![allow(dead_code)]

use postcard_schema::{max_size::max_size, Schema};

#[derive(Schema)]
enum ExampleE {
    Foo,
    Bar([u8; 5]),
    Baz {
        a: Result<i32, u32>,
        b: i128,
    }
}

#[derive(Schema)]
struct ExampleS {
    a: Result<i32, u32>,
    b: i128,
    c: bool,
}

#[test]
fn smoke() {
    let max1 = max_size::<ExampleE>();
    let max2 = max_size::<ExampleS>();
    assert_eq!(Some(34), max1);
    assert_eq!(Some(30), max2);
}
