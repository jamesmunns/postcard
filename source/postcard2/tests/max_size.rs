#![allow(unused_imports)]

#[cfg(feature = "experimental-derive")]
mod tests {
    use postcard2::experimental::max_size::MaxSize;
    use postcard2::to_slice;
    use serde::Serialize;

    #[test]
    fn test_struct_max_size() {
        #[derive(MaxSize)]
        struct Foo {
            _a: u16,
            _b: Option<u8>,
        }

        assert_eq!(Foo::POSTCARD_MAX_SIZE, 5);
    }

    #[test]
    fn test_enum_max_size() {
        #[allow(dead_code)]
        #[derive(MaxSize, Serialize)]
        enum Bar {
            A(u16),
            B(u8),
        }

        assert_eq!(Bar::POSTCARD_MAX_SIZE, 4);
        let mut buf = [0u8; 128];
        let used = to_slice(&Bar::A(0xFFFF), &mut buf).unwrap();
        assert!(
            used.len() <= Bar::POSTCARD_MAX_SIZE,
            "FAIL {} > {}",
            used.len(),
            Bar::POSTCARD_MAX_SIZE
        );

        #[derive(MaxSize)]
        enum Baz {}

        assert_eq!(Baz::POSTCARD_MAX_SIZE, 0);
    }

    #[test]
    fn test_ref() {
        #[allow(dead_code)]
        #[derive(MaxSize)]
        struct Foo {
            a: &'static u32,
        }
    }

    // #[cfg(feature = "experimental-derive")]
    // #[test]
    // fn test_union_max_size() {
    //     #[derive(postcard2::MaxSize)]
    //     union Foo {
    //         a: u16,
    //         b: Option<u8>,
    //     }
    // }

    // #[cfg(feature = "experimental-derive")]
    // #[test]
    // fn test_not_implemented() {
    //     #[derive(postcard2::MaxSize)]
    //     struct Foo {
    //         a: &'static str,
    //     }
    // }
}
