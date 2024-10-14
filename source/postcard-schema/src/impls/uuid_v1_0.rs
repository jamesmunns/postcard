use crate::{
    schema::{NamedType, DataModelType},
    Schema,
};

impl Schema for uuid_v1_0::Uuid {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Uuid",
        ty: &DataModelType::ByteArray,
    };
}
