//! Serializing tools

mod api;
pub mod flavors;

pub use api::{BufferFull, Flavor, Patch};

use crate::varint::{
    varint_max, varint_u16, varint_u32, varint_u64, varint_u128, varint_usize, zig_zag_i16,
    zig_zag_i32, zig_zag_i64, zig_zag_i128,
};

/// Attempt to push a `()`
///
/// Unsurprisingly, as this pushes nothing to `F`, it always succeeds.
// postcard[impl atom.unit.encoding]
#[inline]
pub fn try_push_unit<F: Flavor>(_f: &mut F, _u: ()) -> Result<(), F::PushError> {
    Ok(())
}

/// Attempt to push a [bool]
// postcard[impl atom.bool.encoding]
#[inline]
pub fn try_push_bool<F: Flavor>(f: &mut F, b: bool) -> Result<(), F::PushError> {
    let u = if b { 1 } else { 0 };
    try_push_u8(f, u)
}

/// Attempt to push a [u8]
// postcard[impl atom.u8.encoding]
#[inline]
pub fn try_push_u8<F: Flavor>(f: &mut F, u: u8) -> Result<(), F::PushError> {
    f.try_push(u)
}

/// Attempt to push a [u16]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_push_u16<F: Flavor>(f: &mut F, u: u16) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u16>()];
    let used_buf = varint_u16(u, &mut buf);
    f.try_extend(used_buf)
}

/// Attempt to push a [u32]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_push_u32<F: Flavor>(f: &mut F, u: u32) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u32>()];
    let used_buf = varint_u32(u, &mut buf);
    f.try_extend(used_buf)
}

/// Attempt to push a [u64]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_push_u64<F: Flavor>(f: &mut F, u: u64) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u64>()];
    let used_buf = varint_u64(u, &mut buf);
    f.try_extend(used_buf)
}

/// Attempt to push a [u128]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_push_u128<F: Flavor>(f: &mut F, u: u128) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<u128>()];
    let used_buf = varint_u128(u, &mut buf);
    f.try_extend(used_buf)
}

/// Attempt to push a [usize]
///
/// Encoded as a varint of the platform's pointer width
// postcard[impl atom.usize.encoding]
#[inline]
pub fn try_push_usize<F: Flavor>(f: &mut F, u: usize) -> Result<(), F::PushError> {
    let mut buf = [0u8; varint_max::<usize>()];
    let used_buf = varint_usize(u, &mut buf);
    f.try_extend(used_buf)
}

/// Attempt to push a [i8]
// postcard[impl atom.i8.encoding]
#[inline]
pub fn try_push_i8<F: Flavor>(f: &mut F, i: i8) -> Result<(), F::PushError> {
    let u = i as u8;
    f.try_push(u)
}

/// Attempt to push a [i16]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_push_i16<F: Flavor>(f: &mut F, i: i16) -> Result<(), F::PushError> {
    let u = zig_zag_i16(i);
    try_push_u16(f, u)
}

/// Attempt to push a [i32]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_push_i32<F: Flavor>(f: &mut F, i: i32) -> Result<(), F::PushError> {
    let u = zig_zag_i32(i);
    try_push_u32(f, u)
}

/// Attempt to push a [i64]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_push_i64<F: Flavor>(f: &mut F, i: i64) -> Result<(), F::PushError> {
    let u = zig_zag_i64(i);
    try_push_u64(f, u)
}

/// Attempt to push a [i128]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_push_i128<F: Flavor>(f: &mut F, i: i128) -> Result<(), F::PushError> {
    let u = zig_zag_i128(i);
    try_push_u128(f, u)
}

/// Attempt to push an [isize]
///
/// Zig-zag encoded, then encoded as a varint of the platform's pointer width
// postcard[impl atom.isize.encoding]
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
// postcard[impl atom.f32.encoding]
#[inline]
pub fn try_push_f32<F: Flavor>(f: &mut F, fl: f32) -> Result<(), F::PushError> {
    let u: [u8; 4] = fl.to_bits().to_le_bytes();
    f.try_extend(&u)
}

/// Attempt to push an [f64]
// postcard[impl atom.f64.encoding]
#[inline]
pub fn try_push_f64<F: Flavor>(f: &mut F, fl: f64) -> Result<(), F::PushError> {
    let u: [u8; 8] = fl.to_bits().to_le_bytes();
    f.try_extend(&u)
}

/// Attempt to push a `bytes` value
// postcard[impl atom.bytes.encoding]
#[inline]
pub fn try_push_bytes<F: Flavor>(f: &mut F, b: &[u8]) -> Result<(), F::PushError> {
    try_push_length(f, b.len())?;
    f.try_extend(b)
}

/// Attempt to push a `string`
// postcard[impl atom.string.encoding]
#[inline]
pub fn try_push_str<F: Flavor>(f: &mut F, b: &str) -> Result<(), F::PushError> {
    let bytes = b.as_bytes();
    try_push_bytes(f, bytes)
}

/// Attempt to push the discriminant header of an option with no value (`None`)
// postcard[related atom.bool.encoding]
#[inline]
pub fn try_push_option_none<F: Flavor>(f: &mut F) -> Result<(), F::PushError> {
    try_push_bool(f, false)
}

/// Attempt to push the discriminant header of an option with a value (`Some`)
// postcard[related atom.bool.encoding]
#[inline]
pub fn try_push_option_some<F: Flavor>(f: &mut F) -> Result<(), F::PushError> {
    try_push_bool(f, true)
}

/// Attempt to push the discriminant header of a tagged union
// postcard[impl atom.discriminant.encoding]
#[inline]
pub fn try_push_discriminant<F: Flavor>(f: &mut F, v: u32) -> Result<(), F::PushError> {
    try_push_u32(f, v)
}

/// Attempt to push a length header of a sequence, `bytes`, or `string` value
// postcard[impl atom.length.encoding]
#[inline]
pub fn try_push_length<F: Flavor>(f: &mut F, v: usize) -> Result<(), F::PushError> {
    try_push_usize(f, v)
}

/// Attempt to push a [char]
// postcard[impl atom.char.encoding]
#[inline]
pub fn try_push_char<F: Flavor>(f: &mut F, v: char) -> Result<(), F::PushError> {
    let mut buf = [0u8; 4];
    let strsl = v.encode_utf8(&mut buf);
    try_push_str(f, strsl)
}
