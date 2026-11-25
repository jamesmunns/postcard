//! The key hasher, as defined by the Keys chapter of the postcard-schema
//! specification. Each `schema[impl ...]` comment names the requirement that
//! the code below it implements.

use crate::schema::{Data, DataModelType, NamedField, Variant};

/// FNV-1a 128-bit offset basis
const BASIS: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
/// FNV-1a 128-bit prime, 2^88 + 0x13B
const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

/// The first bytes of every key input
// schema[impl key.input.domain]
const DOMAIN: &[u8] = b"postcard-schema/key/v3";

/// Bits of the options, written as a `varint(u64)` after [`DOMAIN`]
mod flag {
    pub const NAMES: u8 = 1 << 0;
    pub const MAX_LEN: u8 = 1 << 1;
}

/// The tags written before items, types and max lengths
///
/// Each group of tags only has to be unique within its group, as the position
/// in the key input always determines which group is being read.
mod tag {
    // schema[impl key.input.item]
    pub const ITEM_STRING: u8 = 0x00;
    pub const ITEM_SCHEMA: u8 = 0x01;

    // schema[impl key.input.postcard]
    pub const BOOL: u8 = 0x00;
    pub const I8: u8 = 0x01;
    pub const U8: u8 = 0x02;
    pub const I16: u8 = 0x03;
    pub const I32: u8 = 0x04;
    pub const I64: u8 = 0x05;
    pub const I128: u8 = 0x06;
    pub const U16: u8 = 0x07;
    pub const U32: u8 = 0x08;
    pub const U64: u8 = 0x09;
    pub const U128: u8 = 0x0A;
    pub const F32: u8 = 0x0B;
    pub const F64: u8 = 0x0C;
    pub const CHAR: u8 = 0x0D;
    pub const STRING: u8 = 0x0E;
    pub const UNIT: u8 = 0x0F;
    pub const OPTION: u8 = 0x10;
    pub const SEQUENCE: u8 = 0x11;
    pub const AGGREGATE: u8 = 0x12;
    pub const TAGGED_UNION: u8 = 0x13;
    pub const SCHEMA: u8 = 0x14;

    // schema[impl key.input.postcard]
    pub const NONE: u8 = 0x00;
    pub const SOME: u8 = 0x01;
}

/// Options for key hashing
///
/// [`Options::DEFAULT`] enables the names option and disables the maximum
/// lengths option. Options are part of the key, so both sides need to use the
/// same ones.
// schema[impl key.options]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Options {
    /// The names option: hash the names of fields and variants
    pub names: bool,
    /// The maximum lengths option: hash the maximum lengths of strings, byte
    /// arrays, sequences and maps
    pub max_len: bool,
}

impl Options {
    /// The default options, usable in const context
    pub const DEFAULT: Self = Options {
        names: true,
        max_len: false,
    };

    /// Set [`Options::names`]
    pub const fn with_names(mut self, names: bool) -> Self {
        self.names = names;
        self
    }

    /// Set [`Options::max_len`]
    pub const fn with_max_len(mut self, max_len: bool) -> Self {
        self.max_len = max_len;
        self
    }

    /// The options of the key input, as bits
    // schema[impl key.input.options]
    const fn flags(&self) -> u8 {
        let mut flags = 0;
        if self.names {
            flags |= flag::NAMES;
        }
        if self.max_len {
            flags |= flag::MAX_LEN;
        }
        flags
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A hashable item, used for hashing a series of items together at once
// schema[impl key.item]
#[non_exhaustive]
pub enum Item<'a> {
    /// A type's schema
    Schema(&'a DataModelType),
    /// A string
    String(&'a str),
}

/// The state of the hash, passed by value so it can be used in const fns
#[derive(Clone, Copy)]
struct State(u128);

impl State {
    // schema[impl key.input.postcard]
    const fn new(opt: &Options, items: usize) -> Self {
        State(BASIS)
            .bytes(DOMAIN)
            .len(opt.flags() as usize)
            .len(items)
    }

    // schema[impl key.hash.fnv1a-128]
    const fn byte(self, b: u8) -> Self {
        State((self.0 ^ b as u128).wrapping_mul(PRIME))
    }

    const fn bytes(mut self, bytes: &[u8]) -> Self {
        let mut i = 0;
        while i < bytes.len() {
            self = self.byte(bytes[i]);
            i += 1;
        }
        self
    }

    /// A length or count, as a `varint(u64)`
    // schema[impl key.input.postcard]
    const fn len(self, n: usize) -> Self {
        let mut state = self;
        let mut value = n as u64;
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                return state.byte(byte);
            }
            state = state.byte(byte | 0x80);
        }
    }

    // schema[impl key.input.postcard]
    const fn str(self, s: &str) -> Self {
        self.len(s.len()).bytes(s.as_bytes())
    }

    /// Writes nothing unless [`Options::max_len`] is set
    // schema[impl key.input.max-len]
    const fn max_len(self, opt: &Options, max_len: Option<usize>) -> Self {
        if !opt.max_len {
            return self;
        }
        match max_len {
            None => self.byte(tag::NONE),
            Some(n) => self.byte(tag::SOME).len(n),
        }
    }

    /// Writes nothing unless [`Options::names`] is set
    // schema[impl key.input.name]
    const fn name(self, opt: &Options, name: Option<&str>) -> Self {
        if !opt.names {
            return self;
        }
        match name {
            None => self.byte(tag::NONE),
            Some(s) => self.byte(tag::SOME).str(s),
        }
    }

    /// Fold the two halves together, then mix with MurmurHash3's `fmix64`.
    /// The fold keeps information from all 128 bits of state, but a
    /// change in the last input bytes still reaches only a few bits of the
    /// result. `fmix64` is a bijection, so it adds no collisions, and it
    /// spreads every bit over the whole key, so shortened keys (such as
    /// postcard-rpc's `Key1`, `Key2` and `Key4`) stay well distributed.
    // schema[impl key.hash.fold]
    // schema[impl key.hash.fmix64]
    // schema[impl key.hash.output]
    const fn finish(self) -> [u8; 8] {
        let mut k = (self.0 as u64) ^ ((self.0 >> 64) as u64);
        k ^= k >> 33;
        k = k.wrapping_mul(0xff51_afd7_ed55_8ccd);
        k ^= k >> 33;
        k = k.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        k ^= k >> 33;
        k.to_le_bytes()
    }
}

/// Defines `hasher` and the private functions that write a schema into the key
/// input, for one family of schema types: the borrowed [`DataModelType`] or the
/// owned [`OwnedDataModelType`](crate::schema::OwnedDataModelType). Both
/// families must produce the same key input for the same schema, so they share
/// this one definition.
macro_rules! key_hasher {
    ($item:ident, $dmt:ident, $data:ident, $variant:ident, $field:ident) => {
        /// Hash an arbitrary sequence of items in the order provided
        // schema[impl key.items]
        pub(crate) const fn hasher(opt: &Options, items: &[$item<'_>]) -> [u8; 8] {
            let mut state = State::new(opt, items.len());
            let mut idx = 0;
            while idx < items.len() {
                state = match items[idx] {
                    $item::String(s) => state.byte(tag::ITEM_STRING).str(s),
                    $item::Schema(ty) => hash_ty(opt, state.byte(tag::ITEM_SCHEMA), ty),
                };
                idx += 1;
            }
            state.finish()
        }

        // schema[impl key.input.postcard]
        // schema[impl key.input.bytes]
        // schema[impl key.input.map]
        // schema[impl key.input.tuple]
        // schema[impl key.input.enum]
        const fn hash_ty(opt: &Options, state: State, ty: &$dmt) -> State {
            match ty {
                $dmt::Bool => state.byte(tag::BOOL),
                $dmt::I8 => state.byte(tag::I8),
                $dmt::U8 => state.byte(tag::U8),
                $dmt::I16 => state.byte(tag::I16),
                $dmt::I32 => state.byte(tag::I32),
                $dmt::I64 => state.byte(tag::I64),
                $dmt::I128 => state.byte(tag::I128),
                $dmt::U16 => state.byte(tag::U16),
                $dmt::U32 => state.byte(tag::U32),
                $dmt::U64 => state.byte(tag::U64),
                $dmt::U128 => state.byte(tag::U128),
                $dmt::F32 => state.byte(tag::F32),
                $dmt::F64 => state.byte(tag::F64),
                $dmt::Char => state.byte(tag::CHAR),
                $dmt::String { max_len } => state.byte(tag::STRING).max_len(opt, *max_len),
                $dmt::ByteArray { max_len } => state
                    .byte(tag::SEQUENCE)
                    .max_len(opt, *max_len)
                    .byte(tag::U8),
                $dmt::Option(t) => hash_ty(opt, state.byte(tag::OPTION), t),
                $dmt::Unit => state.byte(tag::UNIT),
                $dmt::Seq { element, max_len } => {
                    let state = state.byte(tag::SEQUENCE).max_len(opt, *max_len);
                    hash_ty(opt, state, element)
                }
                $dmt::Tuple(ts) => {
                    let mut state = state.byte(tag::AGGREGATE).len(ts.len());
                    let mut idx = 0;
                    while idx < ts.len() {
                        state = hash_ty(opt, state.name(opt, None), &ts[idx]);
                        idx += 1;
                    }
                    state
                }
                $dmt::Map { key, val, max_len } => {
                    let state = state.byte(tag::SEQUENCE).max_len(opt, *max_len);
                    let state = state.byte(tag::AGGREGATE).len(2);
                    let state = hash_ty(opt, state.name(opt, None), key);
                    hash_ty(opt, state.name(opt, None), val)
                }
                $dmt::Struct { name: _, data } => hash_data(opt, state, data),
                $dmt::Enum { name: _, variants } => {
                    let mut state = state.byte(tag::TAGGED_UNION).len(variants.len());
                    let mut idx = 0;
                    while idx < variants.len() {
                        let v: &$variant = &variants[idx];
                        state = hash_data(opt, state.name(opt, Some(&*v.name)), &v.data);
                        idx += 1;
                    }
                    state
                }
                $dmt::Schema => state.byte(tag::SCHEMA),
            }
        }

        // schema[impl key.input.struct]
        const fn hash_data(opt: &Options, state: State, data: &$data) -> State {
            match data {
                $data::Unit => state.byte(tag::UNIT),
                $data::Newtype(t) => hash_ty(opt, state, t),
                $data::Tuple(ts) => {
                    let mut state = state.byte(tag::AGGREGATE).len(ts.len());
                    let mut idx = 0;
                    while idx < ts.len() {
                        state = hash_ty(opt, state.name(opt, None), &ts[idx]);
                        idx += 1;
                    }
                    state
                }
                $data::Struct(fields) => {
                    let mut state = state.byte(tag::AGGREGATE).len(fields.len());
                    let mut idx = 0;
                    while idx < fields.len() {
                        let f: &$field = &fields[idx];
                        state = hash_ty(opt, state.name(opt, Some(&*f.name)), &f.ty);
                        idx += 1;
                    }
                    state
                }
            }
        }
    };
}

key_hasher!(Item, DataModelType, Data, Variant, NamedField);

#[cfg(any(feature = "std", feature = "alloc"))]
pub(crate) mod owned {
    //! Key hashing for [`OwnedDataModelType`], giving the same keys as the
    //! parent module for the equivalent
    //! [`DataModelType`](crate::schema::DataModelType)

    use super::{tag, Options, State};
    use crate::schema::{OwnedData, OwnedDataModelType, OwnedNamedField, OwnedVariant};

    /// A hashable item, used for hashing a series of items together at once
    ///
    /// An [`Item`](super::Item) whose schema is an [`OwnedDataModelType`].
    #[non_exhaustive]
    pub enum OwnedItem<'a> {
        /// A type's schema
        Schema(&'a OwnedDataModelType),
        /// A string
        String(&'a str),
    }

    key_hasher!(
        OwnedItem,
        OwnedDataModelType,
        OwnedData,
        OwnedVariant,
        OwnedNamedField
    );
}

#[cfg(all(test, feature = "derive", feature = "std"))]
mod tests;
