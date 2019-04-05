use serde::{Serialize, Serializer};

/// A wrapper type that exists as a `usize` at rest, but is serialized
/// to or deserialized from a varint.
pub struct VarintUsize(pub usize);

impl Serialize for VarintUsize
{
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = Self::new_buf();
        let used_buf = self.to_buf(&mut buf);
        serializer.serialize_bytes(used_buf)
    }
}

/// Type alias for the largest buffer needed to store
/// a `usize` varint as bytes
type VarintBuf = [u8; VarintUsize::MAX_BUF_SZ];

impl VarintUsize {
    // Should be 5 for u32, and 10 for u64
    // should probably be something like `ceil((size * 8) / 7)`, not this
    pub const MAX_BUF_SZ: usize = core::mem::size_of::<usize>() + (core::mem::size_of::<usize>() / 4);

    pub fn to_buf<'a, 'b>(&'a self, out: &'b mut VarintBuf) -> &'b mut [u8] {
        let mut value = self.0;
        for i in 0..Self::MAX_BUF_SZ {
            out[i] = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                out[i] |= 0x80;
            } else {
                return &mut out[..=i]
            }
        }
        debug_assert_eq!(value, 0);
        &mut out[..]
    }

    pub const fn new_buf() -> VarintBuf {
        [0u8; Self::MAX_BUF_SZ]
    }
}

