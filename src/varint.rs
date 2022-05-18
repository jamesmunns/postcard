//! Varints
//!
//! This implementation borrows heavily from the `vint64` crate.
//!
//! Below is an example of how prefix bits signal the length of the integer value
//! which follows:
//!
//! | Prefix     | Precision | Total Bytes |
//! |------------|-----------|-------------|
//! | `xxxxxxx1` | 7 bits    | 1 byte      |
//! | `xxxxxx10` | 14 bits   | 2 bytes     |
//! | `xxxxx100` | 21 bits   | 3 bytes     |
//! | `xxxx1000` | 28 bits   | 4 bytes     |
//! | `xxx10000` | 35 bits   | 5 bytes     |
//! | `xx100000` | 42 bits   | 6 bytes     |
//! | `x1000000` | 49 bits   | 7 bytes     |
//! | `10000000` | 56 bits   | 8 bytes     |
//! | `00000000` | 64 bits   | 9 bytes     |

// const USIZE_SIZE: u64 = core::mem::size_of::<usize>();
// const USIZE_SIZE_PLUS_ONE: usize = USIZE_SIZE + 1;

// const fn max_size_header() -> u8 {
//     // 64-bit: 0b0000_0000
//     // 32-bit: 0b0001_0000
//     // 16-bit: 0b0000_0100
//     //  8-bit: 0b0000_0010
//     ((1usize << USIZE_SIZE) & 0xFF) as u8
// }

/// Get the length of an encoded `usize` for the given value in bytes.
#[inline]
pub fn encoded_len_u64(value: u64) -> usize {
    match value.leading_zeros() {
        0..=7 => 9,
        8..=14 => 8,
        15..=21 => 7,
        22..=28 => 6,
        29..=35 => 5,
        36..=42 => 4,
        43..=49 => 3,
        50..=56 => 2,
        57..=64 => 1,
        _ => {
            // SAFETY:
            //
            // The `leading_zeros` intrinsic returns the number of bits that
            // contain a zero bit. The result will always be in the range of
            // 0..=64 for a 64 bit `usize`, so the above pattern is exhaustive, however
            // it is not exhaustive over the return type of `u32`. Because of
            // this, we mark the "uncovered" part of the match as unreachable
            // for performance reasons.
            #[allow(unsafe_code)]
            unsafe {
                core::hint::unreachable_unchecked()
            }
        }
    }
}

#[inline]
pub fn encoded_len_u32(value: u32) -> usize {
    match value.leading_zeros() {
        0..=3 => 5,
        4..=10 => 4,
        11..=17 => 3,
        18..=24 => 2,
        25..=32 => 1,
        _ => {
            // SAFETY:
            //
            // The `leading_zeros` intrinsic returns the number of bits that
            // contain a zero bit. The result will always be in the range of
            // 0..=32 for a 32 bit `usize`, so the above pattern is exhaustive, however
            // it is not exhaustive over the return type of `u32`. Because of
            // this, we mark the "uncovered" part of the match as unreachable
            // for performance reasons.
            #[allow(unsafe_code)]
            unsafe {
                core::hint::unreachable_unchecked()
            }
        }
    }
}

#[inline]
pub fn encoded_len_u16(value: u16) -> usize {
    match value.leading_zeros() {
        0..=1 => 3,
        2..=8 => 2,
        9..=16 => 1,
        _ => {
            // SAFETY:
            //
            // The `leading_zeros` intrinsic returns the number of bits that
            // contain a zero bit. The result will always be in the range of
            // 0..=16 for a 16 bit `usize`, so the above pattern is exhaustive, however
            // it is not exhaustive over the return type of `u32`. Because of
            // this, we mark the "uncovered" part of the match as unreachable
            // for performance reasons.
            #[allow(unsafe_code)]
            unsafe {
                core::hint::unreachable_unchecked()
            }
        }
    }
}

// /// Encode the given usize to the `slice`, using `length` bytes for encoding.
// ///
// /// ## Safety
// ///
// /// * `slice.len()` must be >= `length` or this function will panic
// /// * `length` must be `>= encoded_len(value)` or the value will be truncated
// /// * `length` must be `<= size_of::<usize>() + 1` or the value will be truncated
// #[inline]
// pub fn encode_u64_to_slice(value: usize, length: usize, slice: &mut [u8]) {
//     let header_bytes = &mut slice[..length];

//     if length >= USIZE_SIZE_PLUS_ONE {
//         // In the case where the number of bytes is larger than `usize`,
//         // don't try to encode bits in the header byte, just create the header
//         // and place all of the length bytes in subsequent bytes
//         header_bytes[0] = max_size_header();
//         header_bytes[1..USIZE_SIZE_PLUS_ONE].copy_from_slice(&value.to_le_bytes());
//     } else {
//         let encoded = (value << 1 | 1) << (length - 1);
//         header_bytes.copy_from_slice(&encoded.to_le_bytes()[..length]);
//     }
// }

/// Determine the size of the encoded value (in bytes) based on the
/// encoded header
pub fn decoded_len(byte: u8) -> usize {
    byte.trailing_zeros() as usize + 1
}

// /// Decode an encoded usize.
// ///
// /// Accepts a slice containing the encoded usize.
// pub fn decode_usize(input: &[u8]) -> usize {
//     let length = decoded_len(input[0]);

//     let header_bytes = &input[..length];

//     let mut encoded = [0u8; USIZE_SIZE];

//     if length >= USIZE_SIZE_PLUS_ONE {
//         // usize + 1 special case, see `encode_usize_to_slice()` for details
//         encoded.copy_from_slice(&header_bytes[1..]);
//         usize::from_le_bytes(encoded)
//     } else {
//         encoded[..length].copy_from_slice(header_bytes);
//         usize::from_le_bytes(encoded) >> length
//     }
// }

#[inline]
pub fn devarint_u16(length: usize, data: &[u8; varint_max::<u16>()]) -> u16 {
    let mut buf = [0u8; 2];
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr().add(1), buf.as_mut_ptr(), length - 1);
    }
    let mut value = u16::from_le_bytes(buf);
    value <<= 8 - length;
    value |= (data[0] >> length) as u16;
    value
}

#[inline]
pub fn devarint_u32(length: usize, data: &[u8; varint_max::<u32>()]) -> u32 {
    let mut buf = [0u8; 4];
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr().add(1), buf.as_mut_ptr(), length - 1);
    }
    let mut value = u32::from_le_bytes(buf);
    value <<= 8 - length;
    value |= (data[0] >> length) as u32;
    value
}

#[inline]
pub fn devarint_u64(length: usize, data: &[u8; varint_max::<u64>()]) -> u64 {
    let mut buf = [0u8; 8];
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr().add(1), buf.as_mut_ptr(), length - 1);
    }
    let mut value = u64::from_le_bytes(buf);
    value <<= 8 - length;
    value |= (data[0] >> length) as u64;
    value
}


#[inline]
pub fn varint_u16(mut data: u16, buf: &mut [u8; varint_max::<u16>()]) -> &mut [u8] {
    let length = encoded_len_u16(data);
    let header = ((data << 1 | 1) << (length - 1)) as u8;

    buf[0] = header;
    data >>= 8 - length;
    buf[1..].copy_from_slice(&data.to_le_bytes());

    &mut buf[..length]
}

#[inline]
pub fn varint_u32(mut data: u32, buf: &mut [u8; varint_max::<u32>()]) -> &mut [u8] {
    let length = encoded_len_u32(data);
    let header = ((data << 1 | 1) << (length - 1)) as u8;

    buf[0] = header;
    data >>= 8 - length;
    buf[1..].copy_from_slice(&data.to_le_bytes());

    &mut buf[..length]
}

#[inline]
pub fn varint_u64(mut data: u64, buf: &mut [u8; varint_max::<u64>()]) -> &mut [u8] {
    let length = encoded_len_u64(data);
    let header = ((data << 1 | 1) << (length - 1)) as u8;

    buf[0] = header;
    data >>= 8 - length;
    buf[1..].copy_from_slice(&data.to_le_bytes());

    &mut buf[..length]
}

#[cfg(target_pointer_width = "64")]
pub fn varint_usize(data: usize, buf: &mut [u8; varint_max::<usize>()]) -> &mut [u8] {
    varint_u64(data as u64, buf)
}

#[cfg(target_pointer_width = "32")]
pub fn varint_usize(data: usize, buf: &mut [u8; varint_max::<usize>()]) -> &mut [u8] {
    varint_u32(data as u32, buf)
}

pub const fn varint_max<T: Sized>() -> usize {
    core::mem::size_of::<T>() + 1
}
