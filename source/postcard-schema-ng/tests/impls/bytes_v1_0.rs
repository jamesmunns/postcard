#![cfg(feature = "bytes-v1_0")]

#[test]
fn bytes_round_trip() {
    super::round_trip_test(bytes_v1_0::Bytes::from(vec![1, 2, 3, 4]));
}
