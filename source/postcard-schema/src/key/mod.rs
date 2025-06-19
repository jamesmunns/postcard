//! A hash-based "Key" tag for postcard-schema compatible types
//!
//! This originally was developed for use in postcard-rpc, but has more
//! general use as a general purpose type + usage tag.
//!
//! This key should NOT be relied on for memory safety purposes, validation
//! of the data should still be performed, like that done by serde. It should
//! be treated as misuse **resistant**, not misuse **proof**. It is possible
//! for there to be hash collisions, and as a non-cryptograpic hash, it is
//! likely trivial to intentionally cause a collision.

use serde::{Deserialize, Serialize};

use crate::{schema::DataModelType, Schema};

pub mod hash;

// TODO: https://github.com/knurling-rs/defmt/issues/928
#[cfg(feature = "defmt-v0_3")]
use defmt_v0_3 as defmt;

/// The `Key` uniquely identifies what "kind" of message this is.
///
/// In order to generate it, `postcard-schema` takes two pieces of data:
///
/// * a `&str` "path" URI, similar to how you would use URIs as part of an HTTP path
/// * The schema of the message type itself, using the [`Schema`] trait
///
/// [`Schema`]: crate::Schema
///
/// Specifically, we use [`Fnv1a`](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function),
/// and produce a 64-bit digest, by first hashing the path, then hashing the
/// schema. Fnv1a is a non-cryptographic hash function, designed to be reasonably
/// efficient to compute even on small platforms like microcontrollers.
///
/// Changing **anything** about *either* of the path or the schema will produce
/// a drastically different `Key` value.
#[cfg_attr(feature = "defmt-v0_3", derive(defmt_v0_3::Format))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, Hash)]
pub struct Key([u8; 8]);

impl Schema for Key {
    const SCHEMA: &'static crate::schema::DataModelType = &DataModelType::Struct {
        name: "Key",
        data: crate::schema::Data::Newtype(<[u8; 8] as Schema>::SCHEMA),
    };
}

impl core::fmt::Debug for Key {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Key(")?;
        for b in self.0.iter() {
            f.write_fmt(format_args!("{} ", b))?;
        }
        f.write_str(")")
    }
}

impl Key {
    /// Create a Key for the given type and path
    pub const fn for_path<T>(path: &str) -> Self
    where
        T: Schema + ?Sized,
    {
        Key(hash::fnv1a64::hash_ty_path::<T>(path))
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

#[cfg(feature = "use-std")]
mod key_owned {
    use super::*;
    use crate::schema::owned::OwnedDataModelType;
    impl Key {
        /// Calculate the Key for the given path and [`OwnedDataModelType`]
        pub fn for_owned_schema_path(path: &str, nt: &OwnedDataModelType) -> Key {
            Key(hash::fnv1a64_owned::hash_ty_path_owned(path, nt))
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
}
