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

#[inline]
pub fn varint_usize(n: usize, out: &mut [u8; varint_max::<usize>()]) -> &mut [u8] {
    if n < 128 {
        out[0] = n as u8;
        return &mut out[..1];
    }
    let mut value = n;
    let mut i = 0;
    while value >= 128 {
        out[i] = value.to_le_bytes()[0] | 0b10000000;
        value >>= 7;
        i += 1;
    }
    out[i] = value as u8;
    &mut out[..=i]
}

#[inline]
pub fn varint_u16(n: u16, out: &mut [u8; varint_max::<u16>()]) -> &mut [u8] {
    if n < 128 {
        out[0] = n as u8;
        return &mut out[..1];
    }
    let mut value = n;
    let mut i = 0;
    while value >= 128 {
        out[i] = value.to_le_bytes()[0] | 0b10000000;
        value >>= 7;
        i += 1;
    }
    out[i] = value as u8;
    &mut out[..=i]
}

#[inline]
pub fn varint_u32(n: u32, out: &mut [u8; varint_max::<u32>()]) -> &mut [u8] {
    if n < 128 {
        out[0] = n as u8;
        return &mut out[..1];
    }
    let mut value = n;
    let mut i = 0;
    while value >= 128 {
        out[i] = value.to_le_bytes()[0] | 0b10000000;
        value >>= 7;
        i += 1;
    }
    out[i] = value as u8;
    &mut out[..=i]
}

#[inline]
pub fn varint_u64(n: u64, out: &mut [u8; varint_max::<u64>()]) -> &mut [u8] {
    if n < 128 {
        out[0] = n as u8;
        return &mut out[..1];
    }
    let mut value = n;
    let mut i = 0;
    while value >= 128 {
        out[i] = value.to_le_bytes()[0] | 0b10000000;
        value >>= 7;
        i += 1;
    }
    out[i] = value as u8;
    &mut out[..=i]
}

#[inline]
pub fn varint_u128(n: u128, out: &mut [u8; varint_max::<u128>()]) -> &mut [u8] {
    if n < 128 {
        out[0] = n as u8;
        return &mut out[..1];
    }
    let mut value = n;
    let mut i = 0;
    while value >= 128 {
        out[i] = value.to_le_bytes()[0] | 0b10000000;
        value >>= 7;
        i += 1;
    }
    out[i] = value as u8;
    &mut out[..=i]
}

/// Returns the number of bytes it takes to encode the given `val` using a postcard varint
pub fn bytes_to_encode(val: impl Into<u128>) -> usize {
    let val: u128 = val.into();
    let mut buf = [0; varint_max::<u128>()];

    varint_u128(val, &mut buf).len()
}

#[cfg(test)]
mod tests {
    use crate::varint::bytes_to_encode;

    #[test]
    fn encode_size_1() {
        assert_eq!(bytes_to_encode(1_u32), 1)
    }

    #[test]
    fn encode_size_127() {
        assert_eq!(bytes_to_encode(127_u32), 1)
    }

    #[test]
    fn encode_size_128() {
        assert_eq!(bytes_to_encode(128_u32), 2)
    }

    #[test]
    fn encode_size_16383() {
        assert_eq!(bytes_to_encode(16383_u32), 2)
    }

    #[test]
    fn encode_size_16384() {
        assert_eq!(bytes_to_encode(16384_u32), 3)
    }

    #[test]
    fn encode_size_65535() {
        assert_eq!(bytes_to_encode(65_535_u32), 3)
    }

    #[test]
    fn encode_size_65536() {
        assert_eq!(bytes_to_encode(65_536_u32), 3)
    }
}
