use crate::{
    schema::{NamedType, SdmTy},
    Schema,
};

impl Schema for uuid::Uuid {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Uuid",
        ty: &SdmTy::ByteArray,
    };
}
