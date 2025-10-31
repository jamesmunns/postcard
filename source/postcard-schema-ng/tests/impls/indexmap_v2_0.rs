#![cfg(feature = "indexmap-v2_0")]

#[test]
fn indexmap_round_trip() {
    super::round_trip_test(indexmap_v2_0::IndexMap::<String, u32>::from([
        ("a".to_string(), 2),
        ("b".to_string(), 3),
    ]));
}

#[test]
fn indexset_round_trip() {
    super::round_trip_test(indexmap_v2_0::IndexSet::<u32>::from([1, 2, 3, 4]));
}
