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
        57..=64 => 1,
        50..=56 => 2,
        43..=49 => 3,
        36..=42 => 4,
        29..=35 => 5,
        22..=28 => 6,
        15..=21 => 7,
        8..=14 => 8,
        0..=7 => 9,
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
        25..=32 => 1,
        18..=24 => 2,
        11..=17 => 3,
        4..=10 => 4,
        0..=3 => 5,
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
        9..=16 => 1,
        2..=8 => 2,
        0..=1 => 3,
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

/// Determine the size of the encoded value (in bytes) based on the
/// encoded header
#[inline]
pub fn decoded_len(byte: u8) -> usize {
    byte.trailing_zeros() as usize + 1
}

#[inline]
pub fn devarint_u16(mut data: DitchU16) -> u16 {
    let length = data.len as u16;
    // TODO, body.word needs to be swizzled to native endian!
    unsafe {
        data.body.word <<= 8 - length;
        data.body.word |= (data.header >> length) as u16;
        data.body.word
    }
}

#[inline]
pub fn devarint_u32(mut data: DitchU32) -> u32 {
    let length = data.len as u32;
    // TODO, body.word needs to be swizzled to native endian!
    unsafe {
        data.body.word <<= 8 - length;
        data.body.word |= (data.header >> length) as u32;
        data.body.word
    }
}

#[inline]
pub fn devarint_u64(mut data: DitchU64) -> u64 {
    let length = data.len as u64;
    // TODO, body.word needs to be swizzled to native endian!
    unsafe {
        data.body.word <<= 8 - length;
        data.body.word |= (data.header >> length) as u64;
        data.body.word
    }
}


#[inline]
pub fn varint_u16(mut data: u16) -> DitchU16 {
    let length = encoded_len_u16(data);
    let header = ((data << 1 | 1) << (length - 1)) as u8;

    unsafe {
        DitchU16 {
            len: length as u8,
            header,
            body: UDitchU16 { word: data >> (8 - length) },
        }
    }
}

#[inline]
pub fn varint_u32(mut data: u32) -> DitchU32 {
    let length = encoded_len_u32(data);
    let header = ((data << 1 | 1) << (length - 1)) as u8;

    unsafe {
        DitchU32 {
            len: length as u8,
            header,
            body: UDitchU32 { word: data >> (8 - length) },
        }
    }
}

#[inline]
pub fn varint_u64(mut data: u64) -> DitchU64 {
    let length = encoded_len_u64(data);
    let header = ((data << 1 | 1) << (length - 1)) as u8;

    unsafe {
        DitchU64 {
            len: length as u8,
            header,
            body: UDitchU64 { word: data >> (8 - length) },
        }
    }
}

#[cfg(target_pointer_width = "64")]
#[inline]
pub fn varint_usize(data: usize) -> DitchUsize {
    varint_u64(data as u64)
}

#[cfg(target_pointer_width = "32")]
#[inline]
pub fn varint_usize(data: usize) -> DitchU16 {
    varint_u32(data as u32)
}

#[cfg(target_pointer_width = "32")]
pub type DitchUsize = DitchU32;

#[cfg(target_pointer_width = "64")]
pub type DitchUsize = DitchU64;

pub const fn varint_max<T: Sized>() -> usize {
    core::mem::size_of::<T>() + 1
}

#[derive(Default)]
pub struct DitchU16 {
    pub len: u8,
    pub header: u8,
    pub body: UDitchU16,
}

pub union UDitchU16 {
    pub bytes: [u8; 2],
    pub word: u16,
}

impl Default for UDitchU16 {
    #[inline]
    fn default() -> Self {
        Self { word: 0 }
    }
}

#[derive(Default)]
pub struct DitchU32 {
    pub len: u8,
    pub header: u8,
    pub body: UDitchU32,
}

pub union UDitchU32 {
    pub bytes: [u8; 4],
    pub word: u32,
}

impl Default for UDitchU32 {
    #[inline]
    fn default() -> Self {
        Self { word: 0 }
    }
}

#[derive(Default)]
pub struct DitchU64 {
    pub len: u8,
    pub header: u8,
    pub body: UDitchU64,
}

pub union UDitchU64 {
    pub bytes: [u8; 8],
    pub word: u64,
}

impl Default for UDitchU64 {
    #[inline]
    fn default() -> Self {
        Self { word: 0 }
    }
}
