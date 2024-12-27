//! Implementations of the [`Schema`] trait for the `chrono` crate v0.4

use crate::{schema::DataModelType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "chrono-v0_4")))]
impl<Tz: chrono_v0_4::TimeZone> Schema for chrono_v0_4::DateTime<Tz> {
    const SCHEMA: &'static DataModelType = &DataModelType::String;
}
