//! Implementations of the [`Schema`] trait for foreign crates
//!
//! Each module requires the matching feature flag to be enabled.

use crate::{
    schema::{DataModelType, NamedType},
    Schema,
};

pub mod builtins_nostd;

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "use-std"))))]
pub mod builtins_alloc;

#[cfg(feature = "use-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "use-std")))]
pub mod builtins_std;

#[cfg(feature = "chrono-v0_4")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono-v0_4")))]
pub mod chrono_v0_4;

#[cfg(feature = "uuid-v1_0")]
#[cfg_attr(docsrs, doc(cfg(feature = "uuid-v1_0")))]
pub mod uuid_v1_0;

#[cfg(feature = "heapless-v0_7")]
#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_7")))]
pub mod heapless_v0_7;

#[cfg(feature = "heapless-v0_8")]
#[cfg_attr(docsrs, doc(cfg(feature = "heapless-v0_8")))]
pub mod heapless_v0_8;

#[cfg(feature = "nalgebra-v0_33")]
#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v0_33")))]
pub mod nalgebra_v0_33;

impl Schema for NamedType {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "NamedType",
        ty: &DataModelType::Schema,
    };
}
