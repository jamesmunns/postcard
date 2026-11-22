//! # Deserialization Flavors
//!
//! "Flavors" in `postcard` are used as modifiers to the serialization or deserialization
//! process. Flavors typically modify one or both of the following:
//!
//! 1. The source medium of the deserialization, e.g. whether the data is serialized from a `[u8]` slice, or some other container
//! 2. The format of the deserialization, such as if the original data is encoded in a COBS format, contains a CRC32 checksum
//!    appended to the message, etc.
//!
//! Flavors are implemented using the [`Flavor`] trait, which acts as a "middleware" for retrieving the bytes before they
//! are passed to `serde` for deserialization
//!
//! Multiple flavors may be combined to obtain a desired combination of behavior and storage.
//! When flavors are combined, it is expected that the storage flavor (such as [`Slice`]) is the innermost flavor.
//!
//! Custom flavors may be defined by users of the `postcard` crate, however some commonly useful flavors have been provided in
//! this module. If you think your custom flavor would be useful to others, PRs adding flavors are very welcome!
//!
//! ## Usability
//!
//! Flavors may not always be convenient to use directly, as they may expose some implementation details of how the
//! inner workings of the flavor behaves. It is typical to provide a convenience method for using a flavor, to prevent
//! the user from having to specify generic parameters, setting correct initialization values, or handling the output of
//! the flavor correctly. See `postcard::from_bytes()` for an example of this.
//!
//! ## When to use (multiple) flavors
//!
//! Combining flavors are nice for convenience, as they perform potentially multiple steps of
//! serialization at one time.
//!
//! This can often be more memory efficient, as intermediate buffers are not typically required.
//!
//! ## When NOT to use (multiple) flavors
//!
//! The downside of passing deserialization through multiple steps is that it is typically slower than
//! performing each step serially. Said simply, "cobs decoding while deserializing" is often slower
//! than "cobs decode then deserialize", due to the ability to handle longer "runs" of data in each
//! stage. The downside is that if these stages can not be performed in-place on the buffer, you
//! will need additional buffers for each stage.
//!
//! Additionally, deserializating flavors can be more restrictive or difficult to work with than
//! serialization flavors, as deserialization may require that the deserialized types borrow some
//! portion of the original message.
//!
//! ## Examples
//!
//! ### Using a single flavor
//!
//! In the first example, we use the `Slice` flavor, to retrieve the serialized output from a `[u8]` slice.
//! No other modification is made to the serialization process.
//!
//! ```rust
//! use postcard::{
//!     de_flavors::Slice,
//!     Deserializer,
//! };
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Debug, PartialEq)]
//! struct Tup(u8, u8, u8);
//!
//! let msg = [0x04, 0x00, 0x04, 0x01, 0x02, 0x03];
//! let slice = Slice::new(&msg);
//! let mut deserializer = Deserializer::from_flavor(slice);
//! let t = Tup::deserialize(&mut deserializer).unwrap();
//! assert_eq!(t, Tup(4, 0, 4));
//! let remainder = deserializer.finalize().unwrap();
//! assert_eq!(remainder, &[1, 2, 3]);
//! ```

pub use postcard_core::de::flavors::Slice;
pub use postcard_core::de::{Flavor, UnexpectedEnd};
