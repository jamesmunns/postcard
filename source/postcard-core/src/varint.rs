//! Varint tools

/// Returns the maximum number of bytes required to encode T.
///
/// This only considers `size_of::<T>()`, and is only expected to be meaningful
/// for the integer types that postcard encodes as varints: `u16`, `u32`, `u64`,
/// `u128`, `usize`, and corresponding signed integers.
///
/// For any other `T` it returns the length of a varint holding `size_of::<T>()`
/// bytes of data, which is not useful, but also not incorrect.
///
/// ```
/// use postcard_core::varint::varint_max;
///
/// assert_eq!(varint_max::<u16>(), 3);
/// assert_eq!(varint_max::<i32>(), 5);
/// assert_eq!(varint_max::<u128>(), 19);
///
/// // Don't do this, it is silly.
/// assert_eq!(varint_max::<[u8; 4]>(), 5);
/// ```
// postcard[impl atom.varint.max-len]
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
// postcard[impl atom.varint.decode.max-value]
pub(crate) const fn max_of_last_byte<T: Sized>() -> u8 {
    let max_bits = core::mem::size_of::<T>() * 8;
    let extra_bits = max_bits % 7;
    (1 << extra_bits) - 1
}

/// The unsigned integer operations needed to encode and decode varints
///
/// This exists to paper over the fact there is no generic way to interact with
/// integers.
pub(crate) trait VarintUnsigned: Copy + PartialEq {
    const ZERO: Self;
    /// Get the current least significant byte of an integer
    fn least_significant_byte(self) -> u8;
    /// Right-shift the integer 7 bits, e.g. `self >> 7`
    fn shift_right_7(self) -> Self;
    /// Is this a terminal byte, e.g. `self < 128`?
    fn is_last_byte(self) -> bool;
    /// Shift self by `shift` and or with the 7 lsb of `byte`
    fn insert_right_7(self, byte: u8, shift: usize) -> Self;
}

macro_rules! impl_varint_unsigned {
    ($($t:ty),*) => {
        $(
            impl VarintUnsigned for $t {
                const ZERO: Self = 0;

                #[inline(always)]
                fn least_significant_byte(self) -> u8 {
                    self.to_le_bytes()[0]
                }

                #[inline(always)]
                fn shift_right_7(self) -> Self {
                    self >> 7
                }

                #[inline(always)]
                fn is_last_byte(self) -> bool {
                    self < 128
                }

                #[inline(always)]
                fn insert_right_7(self, byte: u8, shift: usize) -> Self {
                    self | (((byte & 0x7F) as $t) << shift)
                }
            }
        )*
    };
}

impl_varint_unsigned!(u16, u32, u64, u128, usize);

/// Encode `n` as a varint into `out`, which must be `varint_max::<T>()` bytes long.
// postcard[impl atom.varint.encoding.continuation]
// postcard[impl atom.varint.encoding.little-endian]
// postcard[impl atom.varint.encode.canonical]
#[inline]
fn encode_varint<T: VarintUnsigned>(n: T, out: &mut [u8]) -> &mut [u8] {
    debug_assert_eq!(out.len(), varint_max::<T>());
    let mut value = n;
    for i in 0..out.len() {
        out[i] = value.least_significant_byte();
        if value.is_last_byte() {
            return &mut out[..=i];
        }

        out[i] |= 0x80;
        value = value.shift_right_7();
    }
    debug_assert!(value == T::ZERO);
    out
}

/// Encode a `varint(usize)`
// postcard[related atom.varint.types]
#[inline]
pub fn varint_usize(n: usize, out: &mut [u8; varint_max::<usize>()]) -> &mut [u8] {
    encode_varint(n, out)
}

/// Encode a `varint(u16)`
// postcard[related atom.varint.types]
#[inline]
pub fn varint_u16(n: u16, out: &mut [u8; varint_max::<u16>()]) -> &mut [u8] {
    encode_varint(n, out)
}

/// Encode a `varint(u32)`
// postcard[related atom.varint.types]
#[inline]
pub fn varint_u32(n: u32, out: &mut [u8; varint_max::<u32>()]) -> &mut [u8] {
    encode_varint(n, out)
}

/// Encode a `varint(u64)`
// postcard[related atom.varint.types]
#[inline]
pub fn varint_u64(n: u64, out: &mut [u8; varint_max::<u64>()]) -> &mut [u8] {
    encode_varint(n, out)
}

/// Encode a `varint(u128)`
// postcard[related atom.varint.types]
#[inline]
pub fn varint_u128(n: u128, out: &mut [u8; varint_max::<u128>()]) -> &mut [u8] {
    encode_varint(n, out)
}

/// Convert an `i16` into its zig-zag encoded `u16` counterpart
// postcard[impl atom.varint.encoding.zigzag]
#[inline]
pub fn zig_zag_i16(n: i16) -> u16 {
    ((n << 1) ^ (n >> 15)) as u16
}

/// Convert an `i32` into its zig-zag encoded `u32` counterpart
// postcard[impl atom.varint.encoding.zigzag]
#[inline]
pub fn zig_zag_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

/// Convert an `i64` into its zig-zag encoded `u64` counterpart
// postcard[impl atom.varint.encoding.zigzag]
#[inline]
pub fn zig_zag_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// Convert an `i128` into its zig-zag encoded `u128` counterpart
// postcard[impl atom.varint.encoding.zigzag]
#[inline]
pub fn zig_zag_i128(n: i128) -> u128 {
    ((n << 1) ^ (n >> 127)) as u128
}

/// Convert a zig-zag encoded `i16` from its `u16` counterpart
// postcard[impl atom.varint.encoding.zigzag]
#[inline]
pub fn de_zig_zag_i16(n: u16) -> i16 {
    ((n >> 1) as i16) ^ (-((n & 0b1) as i16))
}

/// Convert a zig-zag encoded `i32` from its `u32` counterpart
// postcard[impl atom.varint.encoding.zigzag]
#[inline]
pub fn de_zig_zag_i32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 0b1) as i32))
}

/// Convert a zig-zag encoded `i64` from its `u64` counterpart
// postcard[impl atom.varint.encoding.zigzag]
#[inline]
pub fn de_zig_zag_i64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 0b1) as i64))
}

/// Convert a zig-zag encoded `i128` from its `u128` counterpart
// postcard[impl atom.varint.encoding.zigzag]
#[inline]
pub fn de_zig_zag_i128(n: u128) -> i128 {
    ((n >> 1) as i128) ^ (-((n & 0b1) as i128))
}
