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

#[inline]
pub fn try_push_bool<F: Flavor>(f: &mut F, b: bool) -> Result<(), F::PushError> {
    let u = if b { 1 } else { 0 };
    try_push_u8(f, u)
}

#[inline]
pub fn try_push_u8<F: Flavor>(f: &mut F, u: u8) -> Result<(), F::PushError> {
    f.try_push(u)
}

#[inline]
pub fn try_push_u16<F: Flavor>(f: &mut F, u: u16) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u16>()];
    let used_buf = varint_u16(u, &mut buf);
    f.try_extend(used_buf)
}

#[inline]
pub fn try_push_u32<F: Flavor>(f: &mut F, u: u32) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u32>()];
    let used_buf = varint_u32(u, &mut buf);
    f.try_extend(used_buf)
}

#[inline]
pub fn try_push_u64<F: Flavor>(f: &mut F, u: u64) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u64>()];
    let used_buf = varint_u64(u, &mut buf);
    f.try_extend(used_buf)
}

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

#[inline]
pub fn try_push_i8<F: Flavor>(f: &mut F, i: i8) -> Result<(), F::PushError> {
    let u = i as u8;
    f.try_push(u)
}

#[inline]
pub fn try_push_i16<F: Flavor>(f: &mut F, i: i16) -> Result<(), F::PushError> {
    let u = zig_zag_i16(i);
    try_push_u16(f, u)
}

#[inline]
pub fn try_push_i32<F: Flavor>(f: &mut F, i: i32) -> Result<(), F::PushError> {
    let u = zig_zag_i32(i);
    try_push_u32(f, u)
}

#[inline]
pub fn try_push_i64<F: Flavor>(f: &mut F, i: i64) -> Result<(), F::PushError> {
    let u = zig_zag_i64(i);
    try_push_u64(f, u)
}

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

#[inline]
pub fn try_push_f32<F: Flavor>(f: &mut F, fl: f32) -> Result<(), F::PushError> {
    let u: [u8; 4] = fl.to_bits().to_le_bytes();
    f.try_extend(&u)
}

#[inline]
pub fn try_push_f64<F: Flavor>(f: &mut F, fl: f64) -> Result<(), F::PushError> {
    let u: [u8; 8] = fl.to_bits().to_le_bytes();
    f.try_extend(&u)
}

#[inline]
pub fn try_push_bytes<F: Flavor>(f: &mut F, b: &[u8]) -> Result<(), F::PushError> {
    try_push_usize(f, b.len())?;
    f.try_extend(b)
}

#[inline]
pub fn try_push_str<F: Flavor>(f: &mut F, b: &str) -> Result<(), F::PushError> {
    let bytes = b.as_bytes();
    try_push_bytes(f, bytes)
}

#[inline]
pub fn try_push_option_none<F: Flavor>(f: &mut F) -> Result<(), F::PushError> {
    try_push_u8(f, 0)
}

#[inline]
pub fn try_push_option_some<F: Flavor>(f: &mut F) -> Result<(), F::PushError> {
    try_push_u8(f, 1)
}
