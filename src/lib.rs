mod de;
mod ser;

pub use de::from_slice_dyn;
pub use ser::to_stdvec_dyn;
pub use serde_json::Value;
