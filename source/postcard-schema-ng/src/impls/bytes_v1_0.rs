//! Implementations of the [`Schema`] trait for the `bytes` crate v1.0

use crate::{schema::DataModelType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "bytes-v1_0")))]
impl Schema for bytes_v1_0::Bytes {
    const SCHEMA: &'static DataModelType = &DataModelType::ByteArray;
}
