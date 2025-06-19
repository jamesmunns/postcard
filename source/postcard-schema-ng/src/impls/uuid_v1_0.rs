//! Implementations of the [`Schema`] trait for the `uuid` crate v1.0

use crate::{schema::DataModelType, Schema};

impl Schema for uuid_v1_0::Uuid {
    const SCHEMA: &'static DataModelType = &DataModelType::ByteArray;
}
