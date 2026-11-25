//! A hash-based "Key" tag for postcard-schema compatible types
//!
//! This originally was developed for use in postcard-rpc, but has more
//! general use as a general purpose type + usage tag.
//!
//! This key should NOT be relied on for memory safety purposes, validation
//! of the data should still be performed, like that done by serde. It should
//! be treated as misuse **resistant**, not misuse **proof**. It is possible
//! for there to be hash collisions, and as a non-cryptographic hash, it is
//! likely trivial to intentionally cause a collision.
//!
//! A [`Key`] is computed from [`Options`] and an ordered list of [`Item`]s.
//! The application chooses both, and both sides of a connection need to choose
//! the same ones. The Keys chapter of the postcard-schema specification defines
//! everything else, including what changes a key.

use serde::{Deserialize, Serialize};

use crate::{schema::DataModelType, Schema};

mod hash;
#[cfg(feature = "key-v2")]
#[cfg_attr(docsrs, doc(cfg(feature = "key-v2")))]
pub mod v2;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use hash::owned::OwnedItem;
pub use hash::{Item, Options};

/// The `Key` uniquely identifies what "kind" of message this is.
///
/// Created with [`Key::new`], or [`Key::new_owned`] for owned schemas.
///
/// A change to a schema changes its key only if it changes the schema's
/// postcard type, or a name or maximum length that the [`Options`] include. A
/// newtype struct and its field, for example, have the same key, and the names
/// of types are never part of a key.
#[cfg_attr(feature = "defmt-v1", derive(defmt::Format))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, Hash)]
pub struct Key([u8; 8]);

impl Schema for Key {
    const SCHEMA: &'static crate::schema::DataModelType = &DataModelType::Struct {
        name: "Key",
        data: crate::schema::Data::Newtype(<[u8; 8] as Schema>::SCHEMA),
    };
}

/// Formats the bytes as lowercase hex, in order, as in `Key(94a18b70c2aa4b3f)`.
impl core::fmt::Debug for Key {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Key({self})")
    }
}

/// Formats the bytes as lowercase hex, in order, as in `94a18b70c2aa4b3f`.
impl core::fmt::Display for Key {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

impl Key {
    /// The key of `items`, with `options`
    pub const fn new(options: &Options, items: &[Item<'_>]) -> Self {
        Key(hash::hasher(options, items))
    }

    /// The version 2 key of `items`, with `options`, as computed by `flavor`,
    /// for talking to peers that still use version 2 (see [`v2`])
    #[cfg(feature = "key-v2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "key-v2")))]
    pub const fn new_v2(flavor: v2::Flavor, options: &Options, items: &[Item<'_>]) -> Self {
        Key(v2::hasher(flavor, options, items))
    }

    /// Create a key from a given 8-byte value
    ///
    /// NOTE: Since [`Key`]s should never be used to replace full type safety,
    /// creating a "wrong" Key should not be unsafe. However, using a "wrong"
    /// key (which doesn't match the type being deserialized) may cause confusion,
    /// so manually creating keys should be avoided whenever possible.
    pub const fn from_bytes(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    /// Extract the bytes making up this key
    pub const fn to_bytes(&self) -> [u8; 8] {
        self.0
    }

    /// Compare 2 keys in const context.
    pub const fn const_cmp(&self, other: &Self) -> bool {
        let mut i = 0;
        while i < self.0.len() {
            if self.0[i] != other.0[i] {
                return false;
            }

            i += 1;
        }

        true
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
mod key_owned {
    use super::*;
    impl Key {
        /// The key of `items`, with `options`, for owned schemas
        ///
        /// This gives the same key as [`Key::new`] for the equivalent borrowed
        /// items.
        pub const fn new_owned(options: &Options, items: &[OwnedItem<'_>]) -> Self {
            Key(hash::owned::hasher(options, items))
        }

        /// The version 2 key of `items`, with `options`, for owned schemas
        /// (see [`v2`])
        #[cfg(feature = "key-v2")]
        #[cfg_attr(docsrs, doc(cfg(feature = "key-v2")))]
        pub const fn new_owned_v2(
            flavor: v2::Flavor,
            options: &Options,
            items: &[OwnedItem<'_>],
        ) -> Self {
            Key(v2::owned::hasher(flavor, options, items))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{key::Key, schema::DataModelType, Schema};

    #[test]
    fn matches_old_postcard_rpc_defn() {
        let old = &DataModelType::Struct {
            name: "Key",
            data: crate::schema::Data::Newtype(&DataModelType::Tuple(&[
                &DataModelType::U8,
                &DataModelType::U8,
                &DataModelType::U8,
                &DataModelType::U8,
                &DataModelType::U8,
                &DataModelType::U8,
                &DataModelType::U8,
                &DataModelType::U8,
            ])),
        };

        let new = <Key as Schema>::SCHEMA;

        assert_eq!(old, new);
    }

    #[test]
    fn formatting() {
        let key = Key::from_bytes([0x94, 0xa1, 0x8b, 0x70, 0xc2, 0xaa, 0x4b, 0x03]);
        assert_eq!(format!("{key}"), "94a18b70c2aa4b03");
        assert_eq!(format!("{key:?}"), "Key(94a18b70c2aa4b03)");
    }
}
