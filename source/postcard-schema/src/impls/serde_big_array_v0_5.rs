//! Implementations of the [`Schema`] trait for the `serde-big-array` crate v0.5.

use crate::{schema::DataModelType, Schema};

use serde_big_array_v0_5::Array;

impl<T: Schema, const N: usize> Schema for Array<T, N> {
    const SCHEMA: &'static DataModelType = <[T; N] as Schema>::SCHEMA;
}
