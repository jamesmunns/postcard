use crate::{
    schema::{NamedType, DataModelType},
    Schema,
};

pub mod builtins_nostd;

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
pub mod builtins_alloc;

#[cfg(feature = "use-std")]
pub mod builtins_std;

#[cfg(feature = "chrono-v0_4")]
pub mod chrono_v0_4;

#[cfg(feature = "uuid-v1_0")]
pub mod uuid_v1_0;

#[cfg(feature = "heapless-v0_7")]
pub mod heapless_v0_7;

#[cfg(feature = "heapless-v0_8")]
pub mod heapless_v0_8;

#[cfg(feature = "nalgebra-v0_33")]
pub mod nalgebra_v0_33;

impl Schema for NamedType {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "NamedType",
        ty: &DataModelType::Schema,
    };
}
