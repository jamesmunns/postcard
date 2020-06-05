/// A wrapper type that exists as a `usize` at rest, but is serialized
/// to or deserialized from a varint.
#[derive(Debug)]
pub struct VarintUsize(pub usize);

/// Type alias for the largest buffer needed to store
/// a `usize` varint as bytes
///
/// NOTE: This size is different depending on your target
/// platform! For 32 bit platforms, this will be [u8; 5].
/// For 64 bit platforms, this will be [u8; 10].
pub type VarintBuf = [u8; VarintUsize::varint_usize_max()];

impl VarintUsize {
    pub fn to_buf<'a, 'b>(&'a self, out: &'b mut VarintBuf) -> &'b mut [u8] {
        let mut value = self.0;
        for i in 0..Self::varint_usize_max() {
            out[i] = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                out[i] |= 0x80;
            } else {
                return &mut out[..=i];
            }
        }
        debug_assert_eq!(value, 0);
        &mut out[..]
    }

    pub const fn new_buf() -> VarintBuf {
        [0u8; Self::varint_usize_max()]
    }

    pub const fn varint_usize_max() -> usize {
        const BITS_PER_BYTE: usize = 8;
        const BITS_PER_VARINT_BYTE: usize = 7;

        // How many data bits do we need for a usize on this platform?
        let bits = core::mem::size_of::<usize>() * BITS_PER_BYTE;

        // We add (BITS_PER_BYTE - 1), to ensure any integer divisions
        // with a remainder will always add exactly one full byte, but
        // an evenly divided number of bits will be the same
        let roundup_bits = bits + (BITS_PER_BYTE - 1);

        // Apply division, using normal "round down" integer division
        roundup_bits / BITS_PER_VARINT_BYTE
    }
}
