use postcard::{from_bytes, Error};

/// Ensure a `char` is decoded only from a string holding exactly one char.
#[test]
fn char_exactly_one() {
    assert_eq!(from_bytes::<char>(&[0x01, b'a']), Ok('a'));
    assert_eq!(
        from_bytes::<char>(&[0x04, 0xF0, 0x9F, 0xA6, 0x80]),
        Ok('🦀')
    );

    for bad in [
        &[0x00][..],                           // empty
        &[0x02, b'a', b'b'],                   // two chars
        &[0x04, b'a', b'b', b'c', b'd'],       // four chars
        &[0x03, b'a', 0xC3, 0x9F],             // two chars of different lengths
        &[0x05, 0xF0, 0x9F, 0xA6, 0x80, 0x00], // longer than any char
        &[0x01, 0xFF],                         // not valid UTF-8
    ] {
        assert_eq!(
            from_bytes::<char>(bad),
            Err(Error::DeserializeBadChar),
            "{bad:02X?}"
        );
    }
}
