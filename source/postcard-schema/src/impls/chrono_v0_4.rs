use crate::{
    Schema,
    schema::NamedType,
};

impl<Tz: chrono::TimeZone> Schema for chrono::DateTime<Tz> {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "DateTime",
        ty: <&str>::SCHEMA.ty,
    };
}
