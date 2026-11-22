#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! [COBS] framing for [`postcard`].
//!
//! Consistent Overhead Byte Stuffing encodes a message into a frame.
//! This is useful on byte streams with no framing of their own, such as a UART.
//!
//! When decoding, extra `0x00` "sync" bytes before a frame are skipped.
//!
//! [COBS]: https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing

mod accumulator;
mod de;
#[cfg(feature = "postcard-schema-v0_5")]
mod schema;
mod ser;

pub use accumulator::{CobsAccumulator, FeedResult};
pub use de::{DeserializeError, TakeFromBytesCobsError, from_bytes_cobs, take_from_bytes_cobs};
pub use ser::{Cobs, FinalizeError, max_frame_len, to_slice_cobs};

#[cfg(feature = "alloc")]
pub use ser::to_vec_cobs;

#[cfg(feature = "postcard-schema-v0_5")]
pub use schema::max_frame_len_of;
