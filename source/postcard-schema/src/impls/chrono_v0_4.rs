use crate::{schema::NamedType, Schema};

impl<Tz: chrono_v0_4::TimeZone> Schema for chrono_v0_4::DateTime<Tz> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "DateTime",
        ty: <&str>::SCHEMA.ty,
    };
}
