#[test]
#[cfg(feature = "crc")]
fn test_crc() {
    use crc::{Crc, CRC_32_ISCSI};
    use postcard::{
        de_flavors::{crc::CrcModifier as DeCrcModifier, Slice as DeSlice},
        ser_flavors::{crc::CrcModifier as SerCrcModifier, Slice as SerSlice},
        serialize_with_flavor, Deserializer,
    };
    use serde::de::Deserialize;

    let data: &[u8] = &[0x01, 0x00, 0x20, 0x30];
    let buffer = &mut [0u8; 32];
    let crc = Crc::<u32>::new(&CRC_32_ISCSI);
    let digest = crc.digest();
    let res =
        serialize_with_flavor(data, SerCrcModifier::new(SerSlice::new(buffer), digest)).unwrap();

    assert_eq!(res, &[0x04, 0x01, 0x00, 0x20, 0x30, 0x8E, 0xC8, 0x1A, 0x37]);

    let digest = crc.digest();
    let flav = DeCrcModifier::new(DeSlice::new(&res), digest);
    let mut deserializer = Deserializer::from_flavor(flav);
    let res = <[u8; 5]>::deserialize(&mut deserializer).unwrap();

    assert_eq!(res, [0x04, 0x01, 0x00, 0x20, 0x30]);

    assert_eq!(deserializer.finalize().unwrap(), &[0u8; 0]);
}
