#![allow(unused_imports)]

#[cfg(feature = "derive")]
#[test]
fn test_struct_max_size() {
    use postcard::MaxSize;

    #[derive(MaxSize)]
    struct Foo {
        _a: u16,
        _b: Option<u8>,
    }

    assert_eq!(Foo::POSTCARD_MAX_SIZE, 4);
}

#[cfg(feature = "derive")]
#[test]
fn test_enum_max_size() {
    use postcard::MaxSize;

    #[allow(dead_code)]
    #[derive(MaxSize)]
    enum Bar {
        A(u16),
        B(u8),
    }

    assert_eq!(Bar::POSTCARD_MAX_SIZE, 3);

    #[derive(MaxSize)]
    enum Baz {}

    assert_eq!(Baz::POSTCARD_MAX_SIZE, 0);
}

// #[cfg(feature = "derive")]
// #[test]
// fn test_union_max_size() {
//     #[derive(postcard::MaxSize)]
//     union Foo {
//         a: u16,
//         b: Option<u8>,
//     }
// }