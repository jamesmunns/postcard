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
