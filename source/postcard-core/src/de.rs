//! Deserializing tools

mod api;
pub mod flavors;

pub use api::{DecodeError, Flavor, UnexpectedEnd};

use crate::varint::{
    VarintUnsigned, de_zig_zag_i16, de_zig_zag_i32, de_zig_zag_i64, de_zig_zag_i128,
    max_of_last_byte, varint_max,
};

/// Attempt to take a `()`
///
/// Unsurprisingly, as this takes nothing from `F`, it always succeeds.
// postcard[impl atom.unit.decode.empty]
#[inline]
pub fn try_take_unit<'de, F: Flavor<'de>>(_f: &mut F) -> Result<(), DecodeError<F::PopError>> {
    Ok(())
}

/// Attempt to take a [bool]
// postcard[impl atom.bool.encoding]
// postcard[impl atom.bool.decode.invalid]
#[inline]
pub fn try_take_bool<'de, F: Flavor<'de>>(f: &mut F) -> Result<bool, DecodeError<F::PopError>> {
    match f.pop().map_err(DecodeError::Pop)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DecodeError::BadBool),
    }
}

/// Attempt to take a [u8]
// postcard[impl atom.u8.encoding]
// postcard[impl atom.u8.decode.any]
#[inline]
pub fn try_take_u8<'de, F: Flavor<'de>>(f: &mut F) -> Result<u8, DecodeError<F::PopError>> {
    f.pop().map_err(DecodeError::Pop)
}

// postcard[impl atom.varint.encoding.continuation]
// postcard[impl atom.varint.encoding.little-endian]
// postcard[impl atom.varint.decode.non-canonical]
// postcard[impl atom.varint.decode.max-len]
// postcard[impl atom.varint.decode.max-value]
#[inline]
fn try_take_varint<'de, T: VarintUnsigned, F: Flavor<'de>>(
    f: &mut F,
) -> Result<T, DecodeError<F::PopError>> {
    let mut out = T::ZERO;
    for i in 0..varint_max::<T>() {
        let val = f.pop().map_err(DecodeError::Pop)?;
        out = out.insert_right_7(val, 7 * i);

        if (val & 0x80) == 0 {
            if i == varint_max::<T>() - 1 && val > max_of_last_byte::<T>() {
                break;
            } else {
                return Ok(out);
            }
        }
    }
    Err(DecodeError::BadVarint)
}

/// Attempt to take a [u16]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_take_u16<'de, F: Flavor<'de>>(f: &mut F) -> Result<u16, DecodeError<F::PopError>> {
    try_take_varint(f)
}

/// Attempt to take a [u32]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_take_u32<'de, F: Flavor<'de>>(f: &mut F) -> Result<u32, DecodeError<F::PopError>> {
    try_take_varint(f)
}

/// Attempt to take a [u64]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_take_u64<'de, F: Flavor<'de>>(f: &mut F) -> Result<u64, DecodeError<F::PopError>> {
    try_take_varint(f)
}

/// Attempt to take a [u128]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_take_u128<'de, F: Flavor<'de>>(f: &mut F) -> Result<u128, DecodeError<F::PopError>> {
    try_take_varint(f)
}

/// Attempt to take a [usize]
// postcard[impl atom.usize.encoding]
#[inline]
pub fn try_take_usize<'de, F: Flavor<'de>>(f: &mut F) -> Result<usize, DecodeError<F::PopError>> {
    #[cfg(target_pointer_width = "16")]
    let u = try_take_u16(f);

    #[cfg(target_pointer_width = "32")]
    let u = try_take_u32(f);

    #[cfg(target_pointer_width = "64")]
    let u = try_take_u64(f);

    u.map(|u| u as usize)
}

/// Attempt to take a [i8]
// postcard[impl atom.i8.encoding]
// postcard[impl atom.i8.decode.any]
#[inline]
pub fn try_take_i8<'de, F: Flavor<'de>>(f: &mut F) -> Result<i8, DecodeError<F::PopError>> {
    let u = try_take_u8(f)?;
    Ok(u as i8)
}

/// Attempt to take a [i16]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_take_i16<'de, F: Flavor<'de>>(f: &mut F) -> Result<i16, DecodeError<F::PopError>> {
    try_take_u16(f).map(de_zig_zag_i16)
}

/// Attempt to take a [i32]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_take_i32<'de, F: Flavor<'de>>(f: &mut F) -> Result<i32, DecodeError<F::PopError>> {
    try_take_u32(f).map(de_zig_zag_i32)
}

/// Attempt to take a [i64]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_take_i64<'de, F: Flavor<'de>>(f: &mut F) -> Result<i64, DecodeError<F::PopError>> {
    try_take_u64(f).map(de_zig_zag_i64)
}

/// Attempt to take a [i128]
// postcard[impl atom.varint.types]
#[inline]
pub fn try_take_i128<'de, F: Flavor<'de>>(f: &mut F) -> Result<i128, DecodeError<F::PopError>> {
    try_take_u128(f).map(de_zig_zag_i128)
}

/// Attempt to take an [isize]
// postcard[impl atom.isize.encoding]
#[inline]
pub fn try_take_isize<'de, F: Flavor<'de>>(f: &mut F) -> Result<isize, DecodeError<F::PopError>> {
    #[cfg(target_pointer_width = "16")]
    {
        try_take_i16(f).map(|i| i as isize)
    }
    #[cfg(target_pointer_width = "32")]
    {
        try_take_i32(f).map(|i| i as isize)
    }
    #[cfg(target_pointer_width = "64")]
    {
        try_take_i64(f).map(|i| i as isize)
    }
}

/// Attempt to take an [f32]
// postcard[impl atom.f32.encoding]
// postcard[impl atom.f32.decode.any]
// postcard[impl atom.f32.decode.nan]
#[inline]
pub fn try_take_f32<'de, F: Flavor<'de>>(f: &mut F) -> Result<f32, DecodeError<F::PopError>> {
    let bytes = f.try_take_n_temp(4).map_err(DecodeError::Pop)?;
    let mut buf = [0u8; 4];
    buf.copy_from_slice(bytes);
    Ok(f32::from_bits(u32::from_le_bytes(buf)))
}

/// Attempt to take an [f64]
// postcard[impl atom.f64.encoding]
// postcard[impl atom.f64.decode.any]
// postcard[impl atom.f64.decode.nan]
#[inline]
pub fn try_take_f64<'de, F: Flavor<'de>>(f: &mut F) -> Result<f64, DecodeError<F::PopError>> {
    let bytes = f.try_take_n_temp(8).map_err(DecodeError::Pop)?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    Ok(f64::from_bits(u64::from_le_bytes(buf)))
}

/// Attempt to take a `bytes` value, borrowed for the source's `'de` lifetime.
// postcard[impl atom.bytes.encoding]
#[inline]
pub fn try_take_bytes<'de, F: Flavor<'de>>(
    f: &mut F,
) -> Result<&'de [u8], DecodeError<F::PopError>> {
    let len = try_take_length(f)?;
    f.try_take_n(len).map_err(DecodeError::Pop)
}

/// Attempt to take a `bytes` value
///
/// This is used for items that are not required to retain a borrow their data from the source.
// postcard[impl atom.bytes.encoding]
#[inline]
pub fn try_take_bytes_temp<'a, 'de: 'a, F: Flavor<'de>>(
    f: &'a mut F,
) -> Result<&'a [u8], DecodeError<F::PopError>> {
    let len = try_take_length(f)?;
    f.try_take_n_temp(len).map_err(DecodeError::Pop)
}

/// Attempt to take a `string`
// postcard[impl atom.string.encoding]
// postcard[impl atom.string.decode.utf8]
#[inline]
pub fn try_take_str<'de, F: Flavor<'de>>(f: &mut F) -> Result<&'de str, DecodeError<F::PopError>> {
    let sli = try_take_bytes(f)?;
    core::str::from_utf8(sli).map_err(|_| DecodeError::BadUtf8)
}

/// Attempt to take a `string`
///
/// NOTE: This is used for items that are not required to retain a borrow their data from the source.
// postcard[impl atom.string.encoding]
// postcard[impl atom.string.decode.utf8]
#[inline]
pub fn try_take_str_temp<'a, 'de: 'a, F: Flavor<'de>>(
    f: &'a mut F,
) -> Result<&'a str, DecodeError<F::PopError>> {
    let sli = try_take_bytes_temp(f)?;
    core::str::from_utf8(sli).map_err(|_| DecodeError::BadUtf8)
}

/// Attempt to take a [char]
// postcard[impl atom.char.encoding]
// postcard[impl atom.char.decode.one-scalar]
#[inline]
pub fn try_take_char<'de, F: Flavor<'de>>(f: &mut F) -> Result<char, DecodeError<F::PopError>> {
    let sz = try_take_length(f)?;
    if sz > 4 {
        return Err(DecodeError::BadChar);
    }
    let bytes: &[u8] = f.try_take_n_temp(sz).map_err(DecodeError::Pop)?;

    // we pass the character through string conversion because
    // this handles transforming the array of code units to a
    // codepoint. we can't use char::from_u32() because it expects
    // an already-processed codepoint.
    let Ok(strsl) = core::str::from_utf8(bytes) else {
        return Err(DecodeError::BadChar);
    };

    match strsl.chars().next() {
        Some(c) if c.len_utf8() == bytes.len() => Ok(c),
        _ => Err(DecodeError::BadChar),
    }
}

/// Attempt to take the discriminant header of an option.
///
/// `true` indicates that a value follows the `bool`.
// postcard[related atom.bool.encoding]
#[inline]
pub fn try_take_option_discrim<'de, F: Flavor<'de>>(
    f: &mut F,
) -> Result<bool, DecodeError<F::PopError>> {
    try_take_bool(f).map_err(|e| match e {
        DecodeError::BadBool => DecodeError::BadOption,
        e => e,
    })
}

/// Attempt to take a discriminant.
///
/// A discriminant is the header of a tagged union, which identifies which
/// variant's value follows the discriminant.
// postcard[impl atom.discriminant.encoding]
#[inline]
pub fn try_take_discriminant<'de, F: Flavor<'de>>(
    f: &mut F,
) -> Result<u32, DecodeError<F::PopError>> {
    try_take_u32(f)
}

/// Attempt to take a length.
///
/// The length is the header of a sequence, `bytes`, or `string` value.
// postcard[impl atom.length.encoding]
#[inline]
pub fn try_take_length<'de, F: Flavor<'de>>(f: &mut F) -> Result<usize, DecodeError<F::PopError>> {
    try_take_usize(f)
}
