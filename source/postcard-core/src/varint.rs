//! Varint tools

/// Returns the maximum number of bytes required to encode T.
pub const fn varint_max<T: Sized>() -> usize {
    const BITS_PER_BYTE: usize = 8;
    const BITS_PER_VARINT_BYTE: usize = 7;

    // How many data bits do we need for this type?
    let bits = core::mem::size_of::<T>() * BITS_PER_BYTE;

    // We add (BITS_PER_VARINT_BYTE - 1), to ensure any integer divisions
    // with a remainder will always add exactly one full byte, but
    // an evenly divided number of bits will be the same
    let roundup_bits = bits + (BITS_PER_VARINT_BYTE - 1);

    // Apply division, using normal "round down" integer division
    roundup_bits / BITS_PER_VARINT_BYTE
}

/// Returns the maximum value stored in the last encoded byte.
pub const fn max_of_last_byte<T: Sized>() -> u8 {
    let max_bits = core::mem::size_of::<T>() * 8;
    let extra_bits = max_bits % 7;
    (1 << extra_bits) - 1
}

/// Encode a `varint(usize)`
#[inline]
pub fn varint_usize(n: usize, out: &mut [u8; varint_max::<usize>()]) -> &mut [u8] {
    let mut value = n;
    for i in 0..varint_max::<usize>() {
        out[i] = value.to_le_bytes()[0];
        if value < 128 {
            return &mut out[..=i];
        }

        out[i] |= 0x80;
        value >>= 7;
    }
    debug_assert_eq!(value, 0);
    &mut out[..]
}

/// Encode a `varint(u16)`
#[inline]
pub fn varint_u16(n: u16, out: &mut [u8; varint_max::<u16>()]) -> &mut [u8] {
    let mut value = n;
    for i in 0..varint_max::<u16>() {
        out[i] = value.to_le_bytes()[0];
        if value < 128 {
            return &mut out[..=i];
        }

        out[i] |= 0x80;
        value >>= 7;
    }
    debug_assert_eq!(value, 0);
    &mut out[..]
}

/// Encode a `varint(u32)`
#[inline]
pub fn varint_u32(n: u32, out: &mut [u8; varint_max::<u32>()]) -> &mut [u8] {
    let mut value = n;
    for i in 0..varint_max::<u32>() {
        out[i] = value.to_le_bytes()[0];
        if value < 128 {
            return &mut out[..=i];
        }

        out[i] |= 0x80;
        value >>= 7;
    }
    debug_assert_eq!(value, 0);
    &mut out[..]
}

/// Encode a `varint(u64)`
#[inline]
pub fn varint_u64(n: u64, out: &mut [u8; varint_max::<u64>()]) -> &mut [u8] {
    let mut value = n;
    for i in 0..varint_max::<u64>() {
        out[i] = value.to_le_bytes()[0];
        if value < 128 {
            return &mut out[..=i];
        }

        out[i] |= 0x80;
        value >>= 7;
    }
    debug_assert_eq!(value, 0);
    &mut out[..]
}

/// Encode a `varint(u128)`
#[inline]
pub fn varint_u128(n: u128, out: &mut [u8; varint_max::<u128>()]) -> &mut [u8] {
    let mut value = n;
    for i in 0..varint_max::<u128>() {
        out[i] = value.to_le_bytes()[0];
        if value < 128 {
            return &mut out[..=i];
        }

        out[i] |= 0x80;
        value >>= 7;
    }
    debug_assert_eq!(value, 0);
    &mut out[..]
}

/// Convert an `i16` into its zig-zag encoded `u16` counterpart
#[inline]
pub fn zig_zag_i16(n: i16) -> u16 {
    ((n << 1) ^ (n >> 15)) as u16
}

/// Convert an `i32` into its zig-zag encoded `u32` counterpart
#[inline]
pub fn zig_zag_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

/// Convert an `i64` into its zig-zag encoded `u64` counterpart
#[inline]
pub fn zig_zag_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// Convert an `i128` into its zig-zag encoded `u128` counterpart
#[inline]
pub fn zig_zag_i128(n: i128) -> u128 {
    ((n << 1) ^ (n >> 127)) as u128
}

/// Convert a zig-zag encoded `i16` from its `u16` counterpart
#[inline]
pub fn de_zig_zag_i16(n: u16) -> i16 {
    ((n >> 1) as i16) ^ (-((n & 0b1) as i16))
}

/// Convert a zig-zag encoded `i32` from its `u32` counterpart
#[inline]
pub fn de_zig_zag_i32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 0b1) as i32))
}

/// Convert a zig-zag encoded `i64` from its `u64` counterpart
#[inline]
pub fn de_zig_zag_i64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 0b1) as i64))
}

/// Convert a zig-zag encoded `i128` from its `u128` counterpart
#[inline]
pub fn de_zig_zag_i128(n: u128) -> i128 {
    ((n >> 1) as i128) ^ (-((n & 0b1) as i128))
}
