mod bytes_v1_0;

use std::cmp::PartialEq;
use std::fmt::Debug;

use postcard_schema::{schema::owned::OwnedNamedType, Schema};
use serde::{de::DeserializeOwned, Serialize};

fn round_trip_test<T>(val: T)
where
    T: Serialize + DeserializeOwned + Schema + PartialEq + Debug,
{
    let schema = OwnedNamedType::from(T::SCHEMA);
    let encoded = postcard::to_allocvec(&val).unwrap();
    let value = postcard_dyn::from_slice_dyn(&schema, &encoded).unwrap();
    let decoded: T = serde_json::from_value(value).unwrap();
    assert_eq!(val, decoded);
}
