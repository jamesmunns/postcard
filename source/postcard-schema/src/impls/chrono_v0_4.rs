//! Implementations of the [`Schema`] trait for the `chrono` crate v0.4

use crate::{schema::NamedType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "chrono-v0_4")))]
impl<Tz: chrono_v0_4::TimeZone> Schema for chrono_v0_4::DateTime<Tz> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "DateTime",
        ty: <&str>::SCHEMA.ty,
    };
    // TODO: Can we implement manual maxsize for this? The default serialization
    // repr is RFC3339, which I think is bounded, but users can opt into alternative
    // reprs using `serde_with`, see https://docs.rs/chrono/latest/chrono/serde/index.html
    // for more details.
    //
    // A PR is welcome here, if someone needs to calculate max size for chrono types.
}
