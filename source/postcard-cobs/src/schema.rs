use postcard_schema::Schema;

use crate::{CobsAccumulator, max_frame_len};

/// The length of the longest frame that a `T` can encode to, including the terminating
/// `0x00`, for use as the size of an encoding buffer or of a [`CobsAccumulator`].
///
/// ## Example
///
/// ```rust
/// use postcard_cobs::{CobsAccumulator, FeedResult, max_frame_len_of};
/// use postcard_schema::Schema;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, PartialEq, Schema, Serialize, Deserialize)]
/// struct Msg {
///     id: u32,
///     level: Option<u16>,
/// }
///
/// // This can be calculated as a const.
/// const LEN: usize = max_frame_len_of::<Msg>();
/// // Could also be a `static` with a `Mutex` or `static_cell::StaticCell`.
/// let mut acc: CobsAccumulator<LEN> = CobsAccumulator::new();
///
/// // 5 bytes for the `u32`, 1 for the `Option`, 3 for the `u16`, and 2 for COBS
/// assert_eq!(LEN, 11);
///
/// let frame = [0x04, 0x05, 0x01, 0x07, 0x00];
/// assert_eq!(
///     acc.feed::<Msg>(&frame),
///     FeedResult::Success {
///         data: Msg { id: 5, level: Some(7) },
///         remaining: &[],
///     },
/// );
/// ```
///
/// ## Panics
///
/// Panics if `T` has no maximum size, such as a `&str`. In a const context, this
/// is a compile error:
///
/// ```rust,compile_fail
/// use postcard_cobs::{CobsAccumulator, max_frame_len_of};
///
/// static ACC: CobsAccumulator<{ max_frame_len_of::<&str>() }> = CobsAccumulator::new();
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "postcard-schema-v0_5")))]
pub const fn max_frame_len_of<T: Schema + ?Sized>() -> usize {
    match T::SCHEMA.max_size() {
        Some(len) => max_frame_len(len),
        None => panic!("type has no maximum serialized size"),
    }
}

impl<const N: usize> CobsAccumulator<N> {
    /// Create a new accumulator, checking that any `T` fits in a buffer of `N` bytes. If not,
    /// building fails.
    ///
    /// ```rust
    /// use postcard_cobs::CobsAccumulator;
    ///
    /// let acc: CobsAccumulator<7> = CobsAccumulator::new_for::<u32>();
    /// ```
    ///
    /// ```rust,compile_fail
    /// use postcard_cobs::CobsAccumulator;
    ///
    /// // A `u32` can need 5 bytes, and 2 for COBS
    /// let acc: CobsAccumulator<6> = CobsAccumulator::new_for::<u32>();
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "postcard-schema-v0_5")))]
    pub const fn new_for<T: Schema + ?Sized>() -> Self {
        const {
            assert!(
                N >= max_frame_len_of::<T>(),
                "`N` is less than the maximum encoded size of `T`"
            );
        }
        Self::new()
    }
}
