#[test]
#[cfg(feature = "use-crc")]
fn test_crc() {
    use crc::{Crc, CRC_32_ISCSI};

    let data: &[u8] = &[0x01, 0x00, 0x20, 0x30];
    let buffer = &mut [0u8; 32];
    let crc = Crc::<u32>::new(&CRC_32_ISCSI);
    let digest = crc.digest();
    let res = postcard::to_slice_crc32(data, buffer, digest).unwrap();
    assert_eq!(res, &[0x04, 0x01, 0x00, 0x20, 0x30, 0x8E, 0xC8, 0x1A, 0x37]);

    let digest = crc.digest();
    let res = postcard::take_from_bytes_crc32::<[u8; 5]>(&res, digest).unwrap();

    let expected_bytes = [0x04, 0x01, 0x00, 0x20, 0x30];
    let remaining_bytes = [];
    assert_eq!(res, (expected_bytes, remaining_bytes.as_slice()));
}

#[test]
#[cfg(feature = "use-crc")]
fn test_crc_8() {
    use crc::{Crc, CRC_8_SMBUS};

    let data: &[u8] = &[0x01, 0x00, 0x20, 0x30];
    let buffer = &mut [0u8; 32];
    let crc = Crc::<u8>::new(&CRC_8_SMBUS);
    let digest = crc.digest();
    let res = postcard::ser_flavors::crc::to_slice_u8(data, buffer, digest).unwrap();
    assert_eq!(res, &[0x04, 0x01, 0x00, 0x20, 0x30, 167]);

    let digest = crc.digest();
    let res = postcard::de_flavors::crc::take_from_bytes_u8::<[u8; 5]>(&res, digest).unwrap();

    let expected_bytes = [0x04, 0x01, 0x00, 0x20, 0x30];
    let remaining_bytes = [];
    assert_eq!(res, (expected_bytes, remaining_bytes.as_slice()));
}

#[test]
#[cfg(feature = "use-crc")]
fn test_crc_error() {
    use crc::{Crc, CRC_32_ISCSI};

    let data: &[u8] = &[0x01, 0x00, 0x20, 0x30];
    let buffer = &mut [0u8; 32];
    let crc = Crc::<u32>::new(&CRC_32_ISCSI);
    let digest = crc.digest();
    let res = postcard::to_slice_crc32(data, buffer, digest).unwrap();

    // intentionally corrupt the crc
    let last = res.len() - 1;
    res[last] = 0;

    let digest = crc.digest();
    let res = postcard::take_from_bytes_crc32::<[u8; 5]>(&res, digest);

    assert_eq!(res, Err(postcard::Error::DeserializeBadCrc));
}
