//! Version 2 of key hashing, for talking to peers that (still) use it
//!
//! Version 2 is the key hashing of postcard-schema 0.2 (used by postcard-rpc)
//! and postcard-schema-ng 0.4 (used by temperance). It feeds the path, field
//! and variant names, and one tag byte per type into 64-bit FNV-1a, with no
//! lengths or counts, so different schemas can have the same key (see the
//! Versions table of the postcard-schema specification).
//!
//! The two crates differ in a few places, so each function takes a [`Flavor`]
//! saying which one to match. For the same Rust type, a flavor gives the same
//! key as its crate, including where postcard-schema 0.5 describes the
//! type differently:
//!
//! * `fixed` numbers, which both older crates described as their bits;
//! * with [`Flavor::PostcardSchemaV0_2`], `Result`, which 0.2 described as
//!   tuple variants of one field, and `ByteArray`, which has its own tag in
//!   0.2.
//!
//! Some keys can't be matched, as the schema no longer has what the old
//! key was computed from:
//!
//! * `Ipv4Addr`, `Ipv6Addr`, `SocketAddrV4`, and `SocketAddrV6`, which older
//!   crates described as structs with named fields, and `SocketAddrV6` with
//!   two fields that serde never writes;
//! * derived types that use `#[serde(rename)]`, which postcard-derive before
//!   0.2.2 and postcard-derive-ng ignored;
//! * `usize` and `isize`, which postcard-schema 0.2 could describe with their
//!   own kinds (no impl in 0.2 did);
//! * types containing a schema, which older crates described with the `Schema`
//!   kind, such as postcard-rpc's `postcard-rpc/schema/data` topic.
//!
//! For those, use the old key directly with
//! [`Key::from_bytes`](crate::key::Key::from_bytes).
//!
//! [`Options::max_len`] is honoured with [`Flavor::PostcardSchemaNgV0_4`], as
//! `hash_max_size` was, and ignored with [`Flavor::PostcardSchemaV0_2`],
//! which had no maximum lengths. [`Options::names`] is always ignored:
//! version 2 always includes field and variant names.

use crate::key::{Item, Key, Options};
use crate::schema::{Data, DataModelType, NamedField, Variant};

/// Which crate's version 2 keys to match
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    /// postcard-schema 0.2, as used by postcard-rpc
    PostcardSchemaV0_2,
    /// postcard-schema-ng 0.4, as used by temperance
    PostcardSchemaNgV0_4,
}

impl Flavor {
    /// The version 2 key of `items`, with `options`, as this flavor's crate
    /// computes it
    pub const fn key(self, options: &Options, items: &[Item<'_>]) -> Key {
        Key::from_bytes(hasher(self, options, items))
    }

    /// The version 2 key of `items`, with `options`, for owned schemas
    ///
    /// This gives the same key as [`Flavor::key`] for the equivalent borrowed
    /// items.
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub const fn key_owned(self, options: &Options, items: &[crate::key::OwnedItem<'_>]) -> Key {
        Key::from_bytes(owned::hasher(self, options, items))
    }

    const fn is_v0_2(self) -> bool {
        matches!(self, Flavor::PostcardSchemaV0_2)
    }
}

const BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// The tag bytes of version 2: one byte primes, which only have to be unique.
mod tag {
    pub const BOOL: u8 = 0x11;
    pub const I8: u8 = 0xC5;
    pub const U8: u8 = 0x3D;
    pub const I16: u8 = 0x1D;
    pub const I32: u8 = 0x0D;
    pub const I64: u8 = 0x0B;
    pub const I128: u8 = 0x02;
    pub const U16: u8 = 0x83;
    pub const U32: u8 = 0xD3;
    pub const U64: u8 = 0x13;
    pub const U128: u8 = 0x8B;
    pub const F32: u8 = 0xEF;
    pub const F64: u8 = 0x71;
    pub const CHAR: u8 = 0xC1;
    pub const STRING: u8 = 0x25;
    pub const OPTION: u8 = 0x6D;
    pub const UNIT: u8 = 0x47;
    /// Also `ByteArray` in postcard-schema-ng, as a sequence of `u8`
    pub const SEQ: u8 = 0x03;
    pub const TUPLE: u8 = 0xA7;
    pub const MAP: u8 = 0x4F;
    pub const ENUM: u8 = 0xE9;
    pub const MAX_LEN: u8 = 0x2B;

    pub const BYTE_ARRAY_V0_2: u8 = 0x65;

    // The data of a struct
    pub const STRUCT_UNIT: u8 = 0xBF;
    pub const STRUCT_NEWTYPE: u8 = 0x9D;
    pub const STRUCT_TUPLE: u8 = 0x05;
    pub const STRUCT_STRUCT: u8 = 0x7F;
    pub const STRUCT_TAGS: [u8; 4] = [STRUCT_UNIT, STRUCT_NEWTYPE, STRUCT_TUPLE, STRUCT_STRUCT];

    // The data of a variant
    pub const VARIANT_UNIT: u8 = 0xB5;
    pub const VARIANT_NEWTYPE: u8 = 0xDF;
    pub const VARIANT_TUPLE: u8 = 0xC7;
    pub const VARIANT_STRUCT: u8 = 0x67;
    pub const VARIANT_TAGS: [u8; 4] =
        [VARIANT_UNIT, VARIANT_NEWTYPE, VARIANT_TUPLE, VARIANT_STRUCT];
}

const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// The struct names `fixed` serializes with
const fn is_fixed_name(name: &str) -> bool {
    const NAMES: [&str; 10] = [
        "FixedI8",
        "FixedI16",
        "FixedI32",
        "FixedI64",
        "FixedI128",
        "FixedU8",
        "FixedU16",
        "FixedU32",
        "FixedU64",
        "FixedU128",
    ];
    let mut i = 0;
    while i < NAMES.len() {
        if str_eq(name, NAMES[i]) {
            return true;
        }
        i += 1;
    }
    false
}

#[derive(Clone, Copy)]
struct State(u64);

impl State {
    const fn bytes(mut self, bytes: &[u8]) -> Self {
        let mut i = 0;
        while i < bytes.len() {
            self.0 = (self.0 ^ bytes[i] as u64).wrapping_mul(PRIME);
            i += 1;
        }
        self
    }

    const fn byte(self, b: u8) -> Self {
        self.bytes(&[b])
    }

    /// Writes nothing unless `enabled`. The tag is written for `None` too, with
    /// no value after it.
    const fn max_len(self, enabled: bool, max_len: Option<usize>) -> Self {
        if !enabled {
            return self;
        }
        let mut state = self.byte(tag::MAX_LEN);
        let Some(mut value) = max_len else {
            return state;
        };
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                return state.byte(byte);
            }
            state = state.byte(byte | 0x80);
        }
    }
}

/// The settings that every function below passes down
#[derive(Clone, Copy)]
struct Cfg {
    flavor: Flavor,
    max_len: bool,
}

impl Cfg {
    const fn new(flavor: Flavor, opt: &Options) -> Self {
        Cfg {
            flavor,
            max_len: opt.max_len && !flavor.is_v0_2(),
        }
    }
}

/// Version 2 of [`key_hasher!`](super): the same shape, with version 2's tags,
/// no lengths or counts, and the differences of each [`Flavor`].
macro_rules! key_hasher_v2 {
    ($item:ident, $dmt:ident, $data:ident, $variant:ident, $field:ident) => {
        /// Hash a sequence of items with version 2
        pub(crate) const fn hasher(flavor: Flavor, opt: &Options, items: &[$item<'_>]) -> [u8; 8] {
            let cfg = Cfg::new(flavor, opt);
            let mut state = State(BASIS);
            let mut idx = 0;
            while idx < items.len() {
                state = match items[idx] {
                    $item::String(s) => state.bytes(s.as_bytes()),
                    $item::Schema(ty) => hash_ty(cfg, state, ty),
                };
                idx += 1;
            }
            state.0.to_le_bytes()
        }

        const fn hash_ty(cfg: Cfg, state: State, ty: &$dmt) -> State {
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
                $dmt::String { max_len } => state.byte(tag::STRING).max_len(cfg.max_len, *max_len),
                $dmt::ByteArray { .. } if cfg.flavor.is_v0_2() => state.byte(tag::BYTE_ARRAY_V0_2),
                $dmt::ByteArray { max_len } => state
                    .byte(tag::SEQ)
                    .byte(tag::U8)
                    .max_len(cfg.max_len, *max_len),
                $dmt::Option(t) => hash_ty(cfg, state.byte(tag::OPTION), t),
                $dmt::Unit => state.byte(tag::UNIT),
                $dmt::Seq { element, max_len } => {
                    hash_ty(cfg, state.byte(tag::SEQ), element).max_len(cfg.max_len, *max_len)
                }
                $dmt::Tuple(ts) => {
                    let mut state = state.byte(tag::TUPLE);
                    let mut idx = 0;
                    while idx < ts.len() {
                        state = hash_ty(cfg, state, &ts[idx]);
                        idx += 1;
                    }
                    state
                }
                $dmt::Map { key, val, max_len } => {
                    let state = hash_ty(cfg, state.byte(tag::MAP), key);
                    hash_ty(cfg, state, val).max_len(cfg.max_len, *max_len)
                }
                // older crates described `fixed` numbers as their bits
                $dmt::Struct {
                    name,
                    data: $data::Struct(fields),
                } if fields.len() == 1
                    && str_eq(&*fields[0].name, "bits")
                    && is_fixed_name(&*name) =>
                {
                    hash_ty(cfg, state, &fields[0].ty)
                }
                $dmt::Struct { name: _, data } => hash_data(cfg, state, data, tag::STRUCT_TAGS),
                $dmt::Enum { name, variants } => {
                    // postcard-schema 0.2 described `Ok` and `Err` as tuple
                    // variants of one field
                    let result_v0_2 = cfg.flavor.is_v0_2()
                        && str_eq(&*name, "Result")
                        && variants.len() == 2
                        && str_eq(&*variants[0].name, "Ok")
                        && str_eq(&*variants[1].name, "Err")
                        && matches!(variants[0].data, $data::Newtype(_))
                        && matches!(variants[1].data, $data::Newtype(_));
                    let mut state = state.byte(tag::ENUM);
                    let mut idx = 0;
                    while idx < variants.len() {
                        let v: &$variant = &variants[idx];
                        let state_name = state.bytes(v.name.as_bytes());
                        state = match &v.data {
                            $data::Newtype(t) if result_v0_2 => {
                                hash_ty(cfg, state_name.byte(tag::VARIANT_TUPLE), t)
                            }
                            data => hash_data(cfg, state_name, data, tag::VARIANT_TAGS),
                        };
                        idx += 1;
                    }
                    state
                }
            }
        }

        /// `tags` are the unit, newtype, tuple, and struct tags, which differ
        /// between structs and variants.
        const fn hash_data(cfg: Cfg, state: State, data: &$data, tags: [u8; 4]) -> State {
            match data {
                $data::Unit => state.byte(tags[0]),
                $data::Newtype(t) => hash_ty(cfg, state.byte(tags[1]), t),
                $data::Tuple(ts) => {
                    let mut state = state.byte(tags[2]);
                    let mut idx = 0;
                    while idx < ts.len() {
                        state = hash_ty(cfg, state, &ts[idx]);
                        idx += 1;
                    }
                    state
                }
                $data::Struct(fields) => {
                    let mut state = state.byte(tags[3]);
                    let mut idx = 0;
                    while idx < fields.len() {
                        let f: &$field = &fields[idx];
                        state = hash_ty(cfg, state.bytes(f.name.as_bytes()), &f.ty);
                        idx += 1;
                    }
                    state
                }
            }
        }
    };
}

key_hasher_v2!(Item, DataModelType, Data, Variant, NamedField);

#[cfg(any(feature = "std", feature = "alloc"))]
pub(crate) mod owned {
    //! Version 2 key hashing for [`OwnedDataModelType`]

    use super::{is_fixed_name, str_eq, tag, Cfg, Flavor, Options, State, BASIS};
    use crate::key::OwnedItem;
    use crate::schema::{OwnedData, OwnedDataModelType, OwnedNamedField, OwnedVariant};

    key_hasher_v2!(
        OwnedItem,
        OwnedDataModelType,
        OwnedData,
        OwnedVariant,
        OwnedNamedField
    );
}
