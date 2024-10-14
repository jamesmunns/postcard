use crate::{
    Schema,
    schema::{NamedType, SdmTy},
};

impl Schema for uuid::Uuid {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Uuid",
        ty: &SdmTy::ByteArray,
    };
}
