//! Serializing tools

use crate::varint::{
    varint_max, varint_u16, varint_u32, varint_u64, varint_u128, varint_usize, zig_zag_i16,
    zig_zag_i32, zig_zag_i64, zig_zag_i128,
};

/// The serialization buffer is full
#[derive(Debug)]
pub struct BufferFull;

impl core::fmt::Display for BufferFull {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BufferFull")
    }
}

/// The serialization Flavor trait
///
/// This is used as the primary way to encode serialized data into some kind of buffer,
/// or modify that data in a middleware style pattern.
///
/// See the module level docs for an example of how flavors are used.
pub trait Flavor {
    /// The `Output` type is what this storage "resolves" to when the serialization is complete,
    /// such as a slice or a Vec of some sort.
    type Output;

    /// The error type specific to pushing methods.
    ///
    /// This includes [`Self::try_extend`] and [`Self::try_push`].
    ///
    /// If this type cannot error when pushing, e.g. with a `Vec`, consider using
    /// [`Infallible`](core::convert::Infallible). If this type can only fail due
    /// to exhausting available space, consider using [`BufferFull`].
    type PushError: core::fmt::Debug + core::fmt::Display;

    /// The error type specific to [`Self::finalize`].
    ///
    /// If this type cannot error when pushing, e.g. for storage flavors that don't
    /// perform any meaningful finalization actions, consider using
    /// [`Infallible`](core::convert::Infallible).
    type FinalizeError: core::fmt::Debug + core::fmt::Display;

    /// Override this method when you want to customize processing
    /// multiple bytes at once, such as copying a slice to the output,
    /// rather than iterating over one byte at a time.
    #[inline]
    fn try_extend(&mut self, data: &[u8]) -> Result<(), Self::PushError> {
        data.iter().try_for_each(|d| self.try_push(*d))
    }

    /// Push a single byte to be modified and/or stored.
    fn try_push(&mut self, data: u8) -> Result<(), Self::PushError>;

    /// Finalize the serialization process.
    fn finalize(self) -> Result<Self::Output, Self::FinalizeError>;
}

/// Attempt to push a [bool]
///
/// [bool]: https://postcard.jamesmunns.com/wire-format#1---bool
#[inline]
pub fn try_push_bool<F: Flavor>(f: &mut F, b: bool) -> Result<(), F::PushError> {
    let u = if b { 1 } else { 0 };
    try_push_u8(f, u)
}

/// Attempt to push a [u8]
///
/// [u8]: https://postcard.jamesmunns.com/wire-format#7---u8
#[inline]
pub fn try_push_u8<F: Flavor>(f: &mut F, u: u8) -> Result<(), F::PushError> {
    f.try_push(u)
}

/// Attempt to push a [u16]
///
/// [u16]: https://postcard.jamesmunns.com/wire-format#8---u16
#[inline]
pub fn try_push_u16<F: Flavor>(f: &mut F, u: u16) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u16>()];
    let used_buf = varint_u16(u, &mut buf);
    f.try_extend(used_buf)
}

/// Attempt to push a [u32]
///
/// [u32]: https://postcard.jamesmunns.com/wire-format#9---u32
#[inline]
pub fn try_push_u32<F: Flavor>(f: &mut F, u: u32) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u32>()];
    let used_buf = varint_u32(u, &mut buf);
    f.try_extend(used_buf)
}

/// Attempt to push a [u64]
///
/// [u64]: https://postcard.jamesmunns.com/wire-format#10---u64
#[inline]
pub fn try_push_u64<F: Flavor>(f: &mut F, u: u64) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u64>()];
    let used_buf = varint_u64(u, &mut buf);
    f.try_extend(used_buf)
}

/// Attempt to push a [u128]
///
/// [u128]: https://postcard.jamesmunns.com/wire-format#11---u128
#[inline]
pub fn try_push_u128<F: Flavor>(f: &mut F, u: u128) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u128>()];
    let used_buf = varint_u128(u, &mut buf);
    f.try_extend(used_buf)
}

#[inline]
pub fn try_push_usize<F: Flavor>(f: &mut F, u: usize) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<usize>()];
    let used_buf = varint_usize(u, &mut buf);
    f.try_extend(used_buf)
}

/// Attempt to push a [i8]
///
/// [i8]: https://postcard.jamesmunns.com/wire-format#2---i8
#[inline]
pub fn try_push_i8<F: Flavor>(f: &mut F, i: i8) -> Result<(), F::PushError> {
    let u = i as u8;
    f.try_push(u)
}

/// Attempt to push a [i16]
///
/// [i16]: https://postcard.jamesmunns.com/wire-format#3---i16
#[inline]
pub fn try_push_i16<F: Flavor>(f: &mut F, i: i16) -> Result<(), F::PushError> {
    let u = zig_zag_i16(i);
    try_push_u16(f, u)
}

/// Attempt to push a [i32]
///
/// [i32]: https://postcard.jamesmunns.com/wire-format#4---i32
#[inline]
pub fn try_push_i32<F: Flavor>(f: &mut F, i: i32) -> Result<(), F::PushError> {
    let u = zig_zag_i32(i);
    try_push_u32(f, u)
}

/// Attempt to push a [i64]
///
/// [i64]: https://postcard.jamesmunns.com/wire-format#5---i64
#[inline]
pub fn try_push_i64<F: Flavor>(f: &mut F, i: i64) -> Result<(), F::PushError> {
    let u = zig_zag_i64(i);
    try_push_u64(f, u)
}

/// Attempt to push a [i128]
///
/// [i128]: https://postcard.jamesmunns.com/wire-format#6---i128
#[inline]
pub fn try_push_i128<F: Flavor>(f: &mut F, i: i128) -> Result<(), F::PushError> {
    let u = zig_zag_i128(i);
    try_push_u128(f, u)
}

#[inline]
pub fn try_push_isize<F: Flavor>(f: &mut F, i: isize) -> Result<(), F::PushError> {
    #[cfg(target_pointer_width = "16")]
    let u = zig_zag_i16(i as i16) as usize;
    #[cfg(target_pointer_width = "32")]
    let u = zig_zag_i32(i as i32) as usize;
    #[cfg(target_pointer_width = "64")]
    let u = zig_zag_i64(i as i64) as usize;

    try_push_usize(f, u)
}

/// Attempt to push an [f32]
///
/// [f32]: https://postcard.jamesmunns.com/wire-format#12---f32
#[inline]
pub fn try_push_f32<F: Flavor>(f: &mut F, fl: f32) -> Result<(), F::PushError> {
    let u: [u8; 4] = fl.to_bits().to_le_bytes();
    f.try_extend(&u)
}

/// Attempt to push an [f64]
///
/// [f64]: https://postcard.jamesmunns.com/wire-format#13---f64
#[inline]
pub fn try_push_f64<F: Flavor>(f: &mut F, fl: f64) -> Result<(), F::PushError> {
    let u: [u8; 8] = fl.to_bits().to_le_bytes();
    f.try_extend(&u)
}

/// Attempt to push a [byte array]
///
/// [byte array]: https://postcard.jamesmunns.com/wire-format#16---byte-array
#[inline]
pub fn try_push_bytes<F: Flavor>(f: &mut F, b: &[u8]) -> Result<(), F::PushError> {
    try_push_length(f, b.len())?;
    f.try_extend(b)
}

/// Attempt to push a [string]
///
/// [string]: https://postcard.jamesmunns.com/wire-format#15---string
#[inline]
pub fn try_push_str<F: Flavor>(f: &mut F, b: &str) -> Result<(), F::PushError> {
    let bytes = b.as_bytes();
    try_push_bytes(f, bytes)
}

/// Attempt to push an [option] discriminant
///
/// [option]: https://postcard.jamesmunns.com/wire-format#17---option
#[inline]
pub fn try_push_option_none<F: Flavor>(f: &mut F) -> Result<(), F::PushError> {
    try_push_u8(f, 0)
}

/// Attempt to push an [option] discriminant
///
/// [option]: https://postcard.jamesmunns.com/wire-format#17---option
#[inline]
pub fn try_push_option_some<F: Flavor>(f: &mut F) -> Result<(), F::PushError> {
    try_push_u8(f, 1)
}

/// Attempt to push a discriminant
///
/// Used for:
///
/// * [unit variant]
/// * [newtype variant]
/// * [tuple variant]
/// * [struct variant]
///
/// [unit variant]: https://postcard.jamesmunns.com/wire-format#20---unit_variant
/// [newtype variant]: https://postcard.jamesmunns.com/wire-format#22---newtype_variant
/// [tuple variant]: https://postcard.jamesmunns.com/wire-format#26---tuple_variant
/// [struct variant]: https://postcard.jamesmunns.com/wire-format#29---struct_variant
#[inline]
pub fn try_push_discriminant<F: Flavor>(f: &mut F, v: u32) -> Result<(), F::PushError> {
    try_push_u32(f, v)
}

/// Attempt to push a length
///
/// Used for:
///
/// * [seq]
/// * [map]
/// * [byte array]
/// * [string]
///
/// [seq]: https://postcard.jamesmunns.com/wire-format#23---seq
/// [map]: https://postcard.jamesmunns.com/wire-format#27---map
/// [byte array]: https://postcard.jamesmunns.com/wire-format#16---byte-array
/// [string]: https://postcard.jamesmunns.com/wire-format#15---string
#[inline]
pub fn try_push_length<F: Flavor>(f: &mut F, v: usize) -> Result<(), F::PushError> {
    try_push_usize(f, v)
}

/// Attempt to push a [char]
///
/// [char]: https://postcard.jamesmunns.com/wire-format#14---char
#[inline]
pub fn try_push_char<F: Flavor>(f: &mut F, v: char) -> Result<(), F::PushError> {
    let mut buf = [0u8; 4];
    let strsl = v.encode_utf8(&mut buf);
    try_push_str(f, strsl)
}
