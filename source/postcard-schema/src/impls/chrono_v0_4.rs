use crate::{schema::NamedType, Schema};

impl<Tz: chrono::TimeZone> Schema for chrono::DateTime<Tz> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "DateTime",
        ty: <&str>::SCHEMA.ty,
    };
}
