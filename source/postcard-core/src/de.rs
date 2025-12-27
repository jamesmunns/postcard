use crate::varint::{max_of_last_byte, varint_max};

/// Unexpectedly reached the end of the deserialization buffer
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnexpectedEnd;

impl core::fmt::Display for UnexpectedEnd {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("UnexpectedEnd")
    }
}

/// The deserialization Flavor trait
///
/// This is used as the primary way to decode serialized data from some kind of buffer,
/// or modify that data in a middleware style pattern.
///
/// See the module level docs for an example of how flavors are used.
pub trait Flavor<'de>: 'de {
    /// The remaining data of this flavor after deserializing has completed.
    ///
    /// Typically, this includes the remaining buffer that was not used for
    /// deserialization, and in cases of more complex flavors, any additional
    /// information that was decoded or otherwise calculated during
    /// the deserialization process.
    type Remainder: 'de;

    /// The source of data retrieved for deserialization.
    ///
    /// This is typically some sort of data buffer, or another Flavor, when
    /// chained behavior is desired
    type Source: 'de;

    /// The error type specific to pushing methods.
    ///
    /// This includes [`Self::pop`], [`Self::try_take_n`], and [`Self::try_take_n_temp`].
    ///
    /// If the only error is "no more data", consider using [`UnexpectedEnd`].
    type PopError: core::fmt::Debug + core::fmt::Display;

    /// The error type specific to [`Self::finalize`].
    ///
    /// If this type cannot error when pushing, e.g. for storage flavors that don't
    /// perform any meaningful finalization actions, consider using
    /// [`Infallible`](core::convert::Infallible).
    type FinalizeError: core::fmt::Debug + core::fmt::Display;

    /// Obtain the next byte for deserialization
    fn pop(&mut self) -> Result<u8, Self::PopError>;

    /// Returns the number of bytes remaining in the message, if known.
    ///
    /// # Implementation notes
    ///
    /// It is not enforced that this number is exactly correct.
    /// A flavor may yield less or more bytes than the what is hinted at by
    /// this function.
    ///
    /// `size_hint()` is primarily intended to be used for optimizations such as
    /// reserving space for deserialized items, but must not be trusted to
    /// e.g., omit bounds checks in unsafe code. An incorrect implementation of
    /// `size_hint()` should not lead to memory safety violations.
    ///
    /// That said, the implementation should provide a correct estimation,
    /// because otherwise it would be a violation of the trait’s protocol.
    ///
    /// The default implementation returns `None` which is correct for any flavor.
    fn size_hint(&self) -> Option<usize> {
        None
    }

    /// Attempt to take the next `ct` bytes from the serialized message.
    ///
    /// This variant borrows the data from the input for zero-copy deserialization. If zero-copy
    /// deserialization is not necessary, prefer to use `try_take_n_temp` instead.
    fn try_take_n(&mut self, ct: usize) -> Result<&'de [u8], Self::PopError>;

    /// Attempt to take the next `ct` bytes from the serialized message.
    ///
    /// This variant does not guarantee that the returned value is borrowed from the input, so it
    /// cannot be used for zero-copy deserialization, but it also avoids needing to potentially
    /// allocate a data in a temporary buffer.
    ///
    /// This variant should be used instead of `try_take_n`
    /// if zero-copy deserialization is not necessary.
    ///
    /// It is only necessary to implement this method if the flavor requires storing data in a
    /// temporary buffer in order to implement the borrow semantics, e.g. the `std::io::Read`
    /// flavor.
    fn try_take_n_temp<'a>(&'a mut self, ct: usize) -> Result<&'a [u8], Self::PopError>
    where
        'de: 'a,
    {
        self.try_take_n(ct)
    }

    /// Complete the deserialization process.
    ///
    /// This is typically called separately, after the `serde` deserialization
    /// has completed.
    fn finalize(self) -> Result<Self::Remainder, Self::FinalizeError>;
}

#[inline]
pub fn try_take_bool<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<bool>, F::PopError> {
    match f.pop()? {
        0 => Ok(Some(false)),
        1 => Ok(Some(true)),
        _ => Ok(None),
    }
}

#[inline]
pub fn try_take_u8<'de, F: Flavor<'de>>(f: &mut F) -> Result<u8, F::PopError> {
    f.pop()
}

#[inline]
pub fn try_take_u16<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<u16>, F::PopError> {
    let mut out = 0;
    for i in 0..varint_max::<u16>() {
        let val = f.pop()?;
        let carry = (val & 0x7F) as u16;
        out |= carry << (7 * i);

        if (val & 0x80) == 0 {
            if i == varint_max::<u16>() - 1 && val > max_of_last_byte::<u16>() {
                break;
            } else {
                return Ok(Some(out));
            }
        }
    }
    Ok(None)
}

#[inline]
pub fn try_take_u32<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<u32>, F::PopError> {
    let mut out = 0;
    for i in 0..varint_max::<u32>() {
        let val = f.pop()?;
        let carry = (val & 0x7F) as u32;
        out |= carry << (7 * i);

        if (val & 0x80) == 0 {
            if i == varint_max::<u32>() - 1 && val > max_of_last_byte::<u32>() {
                break;
            } else {
                return Ok(Some(out));
            }
        }
    }
    Ok(None)
}

#[inline]
pub fn try_take_u64<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<u64>, F::PopError> {
    let mut out = 0;
    for i in 0..varint_max::<u64>() {
        let val = f.pop()?;
        let carry = (val & 0x7F) as u64;
        out |= carry << (7 * i);

        if (val & 0x80) == 0 {
            if i == varint_max::<u64>() - 1 && val > max_of_last_byte::<u64>() {
                break;
            } else {
                return Ok(Some(out));
            }
        }
    }
    Ok(None)
}

#[inline]
pub fn try_take_u128<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<u128>, F::PopError> {
    let mut out = 0;
    for i in 0..varint_max::<u128>() {
        let val = f.pop()?;
        let carry = (val & 0x7F) as u128;
        out |= carry << (7 * i);

        if (val & 0x80) == 0 {
            if i == varint_max::<u128>() - 1 && val > max_of_last_byte::<u128>() {
                break;
            } else {
                return Ok(Some(out));
            }
        }
    }
    Ok(None)
}

#[inline]
pub fn try_take_usize<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<usize>, F::PopError> {
    #[cfg(target_pointer_width = "16")]
    let u = try_take_u16(f);

    #[cfg(target_pointer_width = "32")]
    let u = try_take_u32(f);

    #[cfg(target_pointer_width = "64")]
    let u = try_take_u64(f);

    u.map(|u| u.map(|u| u as usize))
}

#[inline]
pub fn try_take_i8<'de, F: Flavor<'de>>(f: &mut F) -> Result<i8, F::PopError> {
    let u = try_take_u8(f)?;
    Ok(u as i8)
}

#[inline]
pub fn try_take_i16<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<i16>, F::PopError> {
    let u = try_take_u16(f)?;
    Ok(u.map(de_zig_zag_i16))
}

#[inline]
pub fn try_take_i32<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<i32>, F::PopError> {
    let u = try_take_u32(f)?;
    Ok(u.map(de_zig_zag_i32))
}

#[inline]
pub fn try_take_i64<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<i64>, F::PopError> {
    let u = try_take_u64(f)?;
    Ok(u.map(de_zig_zag_i64))
}

#[inline]
pub fn try_take_i128<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<i128>, F::PopError> {
    let u = try_take_u128(f)?;
    Ok(u.map(de_zig_zag_i128))
}

#[inline]
pub fn try_take_isize<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<isize>, F::PopError> {
    #[cfg(target_pointer_width = "16")]
    {
        let i = try_take_i16(f)?;
        Ok(i.map(|i| i as isize))
    }
    #[cfg(target_pointer_width = "32")]
    {
        let i = try_take_i32(f)?;
        Ok(i.map(|i| i as isize))
    }
    #[cfg(target_pointer_width = "64")]
    {
        let i = try_take_i64(f)?;
        Ok(i.map(|i| i as isize))
    }
}

#[inline]
pub fn try_take_f32<'de, F: Flavor<'de>>(f: &mut F) -> Result<f32, F::PopError> {
    let bytes = f.try_take_n_temp(4)?;
    let mut buf = [0u8; 4];
    buf.copy_from_slice(bytes);
    Ok(f32::from_bits(u32::from_le_bytes(buf)))
}

#[inline]
pub fn try_take_f64<'de, F: Flavor<'de>>(f: &mut F) -> Result<f64, F::PopError> {
    let bytes = f.try_take_n_temp(8)?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    Ok(f64::from_bits(u64::from_le_bytes(buf)))
}

#[inline]
pub fn try_take_bytes<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<&'de [u8]>, F::PopError> {
    let len = match try_take_usize(f) {
        Ok(Some(l)) => l,
        Ok(None) => return Ok(None),
        Err(e) => return Err(e),
    };
    let sli = f.try_take_n(len)?;
    Ok(Some(sli))
}

#[inline]
pub fn try_take_str<'de, F: Flavor<'de>>(f: &mut F) -> Result<Option<&'de str>, F::PopError> {
    let len = match try_take_usize(f) {
        Ok(Some(l)) => l,
        Ok(None) => return Ok(None),
        Err(e) => return Err(e),
    };
    let sli = f.try_take_n(len)?;
    let sli = core::str::from_utf8(sli).ok();
    Ok(sli)
}

pub fn de_zig_zag_i16(n: u16) -> i16 {
    ((n >> 1) as i16) ^ (-((n & 0b1) as i16))
}

pub fn de_zig_zag_i32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 0b1) as i32))
}

pub fn de_zig_zag_i64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 0b1) as i64))
}

pub fn de_zig_zag_i128(n: u128) -> i128 {
    ((n >> 1) as i128) ^ (-((n & 0b1) as i128))
}
