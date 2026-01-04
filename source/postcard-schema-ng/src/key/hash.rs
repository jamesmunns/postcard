//! URI and Schema Hashing
//!
//! We use `FNV1a` hashes with a digest size of 64 bits to represent dispatch keys.
//!
//! Unfortunately. using [core::hash::Hash] seems to not produce consistent results,
//! which [was noted] in the docs. To overcome this, we implement a custom method for
//! hashing the postcard [Schema].
//!
//! [was noted]: https://doc.rust-lang.org/stable/std/hash/trait.Hash.html#portability

use crate::{schema::DataModelType, Schema};

/// A const compatible Fnv1a64 hasher
pub struct Fnv1a64Hasher {
    state: u64,
}

impl Fnv1a64Hasher {
    // source: https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
    const BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    /// Create a new hasher with the default basis as state contents
    pub fn new() -> Self {
        Self { state: Self::BASIS }
    }

    /// Calculate the hash for each of the given data bytes
    pub fn update(&mut self, data: &[u8]) {
        for b in data {
            let ext = u64::from(*b);
            self.state ^= ext;
            self.state = self.state.wrapping_mul(Self::PRIME);
        }
    }

    /// Extract the current state for finalizing the hash
    pub fn digest(self) -> u64 {
        self.state
    }

    /// Same as digest but as bytes
    pub fn digest_bytes(self) -> [u8; 8] {
        self.digest().to_le_bytes()
    }
}

impl Default for Fnv1a64Hasher {
    fn default() -> Self {
        Self::new()
    }
}

pub mod fnv1a64 {
    //! Const and no-std helper methods and types for performing hash calculation

    use crate::schema::{Data, NamedField, Variant};

    use super::*;

    /// Calculate the Key hash for the given path and type T
    pub const fn hash_ty_path<T: Schema + ?Sized>(path: &str) -> [u8; 8] {
        let schema = T::SCHEMA;
        let state = hash_update_str(Fnv1a64Hasher::BASIS, path);
        hash_sdm_type(state, schema).to_le_bytes()
    }

    pub(crate) const fn hash_update(mut state: u64, bytes: &[u8]) -> u64 {
        let mut idx = 0;
        while idx < bytes.len() {
            let ext = bytes[idx] as u64;
            state ^= ext;
            state = state.wrapping_mul(Fnv1a64Hasher::PRIME);
            idx += 1;
        }
        state
    }

    pub(crate) const fn hash_update_str(state: u64, s: &str) -> u64 {
        hash_update(state, s.as_bytes())
    }

    const fn hash_sdm_type(state: u64, sdmty: &'static DataModelType) -> u64 {
        // The actual values we use here don't matter that much (as far as I know),
        // as long as the values for each variant are unique. I am unsure of the
        // implications of doing a TON of single byte calls to `update`, it may be
        // worth doing some buffering, and only calling update every 4/8/16 bytes
        // instead, if performance is a concern.
        //
        // As of initial implementation, I'm mostly concerned with "does it work",
        // as hashing is typically only done on startup.
        //
        // Using all primes that fit into a single byte:
        //
        // all_primes = [
        //     0x02, 0x03, 0x05, 0x07, 0x0B, 0x0D, 0x11, 0x13,
        //     0x17, 0x1D, 0x1F, 0x25, 0x29, 0x2B, 0x2F, 0x35,
        //     0x3B, 0x3D, 0x43, 0x47, 0x49, 0x4F, 0x53, 0x59,
        //     0x61, 0x65, 0x67, 0x6B, 0x6D, 0x71, 0x7F, 0x83,
        //     0x89, 0x8B, 0x95, 0x97, 0x9D, 0xA3, 0xA7, 0xAD,
        //     0xB3, 0xB5, 0xBF, 0xC1, 0xC5, 0xC7, 0xD3, 0xDF,
        //     0xE3, 0xE5, 0xE9, 0xEF, 0xF1, 0xFB,
        // ];
        // shuffled_primes = [
        //     0x11, 0xC5, 0x3D, 0x95, 0x1D, 0x0D, 0x0B, 0x02,
        //     0x83, 0xD3, 0x13, 0x8B, 0x6B, 0xAD, 0xEF, 0x71,
        //     0xC1, 0x25, 0x65, 0x6D, 0x47, 0xBF, 0xB5, 0x9D,
        //     0xDF, 0x03, 0xA7, 0x05, 0xC7, 0x4F, 0x7F, 0x67,
        //     0xE9, 0xB3, 0xE5, 0x2B, 0x97, 0xFB, 0x61, 0x3B,
        //     0x1F, 0xA3, 0x35, 0x43, 0x89, 0x49, 0xE3, 0x07,
        //     0x53, 0xF1, 0x17, 0x2F, 0x29, 0x59,
        // ];
        match sdmty {
            DataModelType::Bool => hash_update(state, &[0x11]),
            DataModelType::I8 => hash_update(state, &[0xC5]),
            DataModelType::U8 => hash_update(state, &[0x3D]),
            DataModelType::I16 => hash_update(state, &[0x1D]),
            DataModelType::I32 => hash_update(state, &[0x0D]),
            DataModelType::I64 => hash_update(state, &[0x0B]),
            DataModelType::I128 => hash_update(state, &[0x02]),
            DataModelType::U16 => hash_update(state, &[0x83]),
            DataModelType::U32 => hash_update(state, &[0xD3]),
            DataModelType::U64 => hash_update(state, &[0x13]),
            DataModelType::U128 => hash_update(state, &[0x8B]),
            DataModelType::Usize => hash_update(state, &[0x6B]),
            DataModelType::Isize => hash_update(state, &[0xAD]),
            DataModelType::F32 => hash_update(state, &[0xEF]),
            DataModelType::F64 => hash_update(state, &[0x71]),
            DataModelType::Char => hash_update(state, &[0xC1]),
            DataModelType::String => hash_update(state, &[0x25]),
            DataModelType::ByteArray => hash_update(state, &[0x65]),
            DataModelType::Option(t) => {
                let state = hash_update(state, &[0x6D]);
                hash_sdm_type(state, t)
            }
            DataModelType::Unit => hash_update(state, &[0x47]),
            DataModelType::Seq(t) => {
                let state = hash_update(state, &[0x03]);
                hash_sdm_type(state, t)
            }
            DataModelType::Tuple(ts) => {
                let mut state = hash_update(state, &[0xA7]);
                let mut idx = 0;
                while idx < ts.len() {
                    state = hash_sdm_type(state, ts[idx]);
                    idx += 1;
                }
                state
            }
            DataModelType::Map { key, val } => {
                let state = hash_update(state, &[0x4F]);
                let state = hash_sdm_type(state, key);
                hash_sdm_type(state, val)
            }
            DataModelType::Struct { name, data } => hash_struct(state, name, data),
            DataModelType::Enum { name: _, variants } => {
                let mut state = hash_update(state, &[0xE9]);
                let mut idx = 0;
                while idx < variants.len() {
                    state = hash_variant(state, variants[idx]);
                    idx += 1;
                }
                state
            }
            DataModelType::Schema => hash_update(state, &[0xE5]),
            DataModelType::Array { item, count } => {
                let mut state = hash_update(state, &[0x2B]);
                let mut idx = 0;

                // TODO: Do we want to hash the len or something? I'm not
                // sure how to handle this for different sizes of `usize`.
                while idx < *count {
                    state = hash_sdm_type(state, item);
                    idx += 1;
                }
                state
            }
        }
    }

    const fn hash_struct(state: u64, _name: &str, data: &Data) -> u64 {
        // NOTE: We do *not* hash the name of the type in hashv2. This
        // is to allow "safe" type punning, e.g. treating `Vec<u8>` and
        // `&[u8]` as compatible types, when talking between std and no-std
        // targets
        //
        // let state = hash_update(state, name.as_bytes());
        match data {
            Data::Unit => hash_update(state, &[0xBF]),
            Data::Newtype(dmt) => {
                let state = hash_update(state, &[0x9D]);
                hash_sdm_type(state, dmt)
            }
            Data::Tuple(dmts) => {
                let mut state = hash_update(state, &[0x05]);
                let mut idx = 0;
                while idx < dmts.len() {
                    state = hash_sdm_type(state, dmts[idx]);
                    idx += 1;
                }
                state
            }
            Data::Struct(nfs) => {
                let mut state = hash_update(state, &[0x7F]);
                let mut idx = 0;
                while idx < nfs.len() {
                    state = hash_named_field(state, nfs[idx]);
                    idx += 1;
                }
                state
            }
        }
    }

    const fn hash_variant(state: u64, nt: &Variant) -> u64 {
        let state = hash_update(state, nt.name.as_bytes());
        match nt.data {
            Data::Unit => hash_update(state, &[0xB5]),
            Data::Newtype(t) => {
                let state = hash_update(state, &[0xDF]);
                hash_sdm_type(state, t)
            }
            Data::Tuple(ts) => {
                let mut state = hash_update(state, &[0xC7]);
                let mut idx = 0;
                while idx < ts.len() {
                    state = hash_sdm_type(state, ts[idx]);
                    idx += 1;
                }
                state
            }
            Data::Struct(fields) => {
                let mut state = hash_update(state, &[0x67]);
                let mut idx = 0;
                while idx < fields.len() {
                    state = hash_named_field(state, fields[idx]);
                    idx += 1;
                }
                state
            }
        }
    }

    const fn hash_named_field(state: u64, nt: &NamedField) -> u64 {
        let state = hash_update(state, nt.name.as_bytes());
        hash_sdm_type(state, nt.ty)
    }
}

#[cfg(feature = "use-std")]
pub mod fnv1a64_owned {
    //! Heapful helpers and versions of hashing for use on `std` targets

    use crate::schema::owned::{OwnedData, OwnedDataModelType, OwnedNamedField, OwnedVariant};

    use super::fnv1a64::*;
    use super::*;

    /// Calculate the Key hash for the given path and [`OwnedDataModelType`]
    pub fn hash_ty_path_owned(path: &str, ty: &OwnedDataModelType) -> [u8; 8] {
        let state = hash_update_str(Fnv1a64Hasher::BASIS, path);
        hash_sdm_type_owned(state, ty).to_le_bytes()
    }

    const fn hash_sdm_type_owned(state: u64, sdmty: &OwnedDataModelType) -> u64 {
        // The actual values we use here don't matter that much (as far as I know),
        // as long as the values for each variant are unique. I am unsure of the
        // implications of doing a TON of single byte calls to `update`, it may be
        // worth doing some buffering, and only calling update every 4/8/16 bytes
        // instead, if performance is a concern.
        //
        // As of initial implementation, I'm mostly concerned with "does it work",
        // as hashing is typically only done on startup.
        //
        // Using all primes that fit into a single byte:
        //
        // all_primes = [
        //     0x02, 0x03, 0x05, 0x07, 0x0B, 0x0D, 0x11, 0x13,
        //     0x17, 0x1D, 0x1F, 0x25, 0x29, 0x2B, 0x2F, 0x35,
        //     0x3B, 0x3D, 0x43, 0x47, 0x49, 0x4F, 0x53, 0x59,
        //     0x61, 0x65, 0x67, 0x6B, 0x6D, 0x71, 0x7F, 0x83,
        //     0x89, 0x8B, 0x95, 0x97, 0x9D, 0xA3, 0xA7, 0xAD,
        //     0xB3, 0xB5, 0xBF, 0xC1, 0xC5, 0xC7, 0xD3, 0xDF,
        //     0xE3, 0xE5, 0xE9, 0xEF, 0xF1, 0xFB,
        // ];
        // shuffled_primes = [
        //     0x11, 0xC5, 0x3D, 0x95, 0x1D, 0x0D, 0x0B, 0x02,
        //     0x83, 0xD3, 0x13, 0x8B, 0x6B, 0xAD, 0xEF, 0x71,
        //     0xC1, 0x25, 0x65, 0x6D, 0x47, 0xBF, 0xB5, 0x9D,
        //     0xDF, 0x03, 0xA7, 0x05, 0xC7, 0x4F, 0x7F, 0x67,
        //     0xE9, 0xB3, 0xE5, 0x2B, 0x97, 0xFB, 0x61, 0x3B,
        //     0x1F, 0xA3, 0x35, 0x43, 0x89, 0x49, 0xE3, 0x07,
        //     0x53, 0xF1, 0x17, 0x2F, 0x29, 0x59,
        // ];
        match sdmty {
            OwnedDataModelType::Bool => hash_update(state, &[0x11]),
            OwnedDataModelType::I8 => hash_update(state, &[0xC5]),
            OwnedDataModelType::U8 => hash_update(state, &[0x3D]),
            OwnedDataModelType::I16 => hash_update(state, &[0x1D]),
            OwnedDataModelType::I32 => hash_update(state, &[0x0D]),
            OwnedDataModelType::I64 => hash_update(state, &[0x0B]),
            OwnedDataModelType::I128 => hash_update(state, &[0x02]),
            OwnedDataModelType::U16 => hash_update(state, &[0x83]),
            OwnedDataModelType::U32 => hash_update(state, &[0xD3]),
            OwnedDataModelType::U64 => hash_update(state, &[0x13]),
            OwnedDataModelType::U128 => hash_update(state, &[0x8B]),
            OwnedDataModelType::Usize => hash_update(state, &[0x6B]),
            OwnedDataModelType::Isize => hash_update(state, &[0xAD]),
            OwnedDataModelType::F32 => hash_update(state, &[0xEF]),
            OwnedDataModelType::F64 => hash_update(state, &[0x71]),
            OwnedDataModelType::Char => hash_update(state, &[0xC1]),
            OwnedDataModelType::String => hash_update(state, &[0x25]),
            OwnedDataModelType::ByteArray => hash_update(state, &[0x65]),
            OwnedDataModelType::Option(t) => {
                let state = hash_update(state, &[0x6D]);
                hash_sdm_type_owned(state, t)
            }
            OwnedDataModelType::Unit => hash_update(state, &[0x47]),
            OwnedDataModelType::Seq(t) => {
                let state = hash_update(state, &[0x03]);
                hash_sdm_type_owned(state, t)
            }
            OwnedDataModelType::Tuple(ts) => {
                let mut state = hash_update(state, &[0xA7]);
                let mut idx = 0;
                while idx < ts.len() {
                    state = hash_sdm_type_owned(state, &ts[idx]);
                    idx += 1;
                }
                state
            }
            OwnedDataModelType::Map { key, val } => {
                let state = hash_update(state, &[0x4F]);
                let state = hash_sdm_type_owned(state, key);
                hash_sdm_type_owned(state, val)
            }
            OwnedDataModelType::Struct { name, data } => hash_struct(state, name, data),
            OwnedDataModelType::Enum { name: _, variants } => {
                let mut state = hash_update(state, &[0xE9]);
                let mut idx = 0;
                while idx < variants.len() {
                    state = hash_variant(state, &variants[idx]);
                    idx += 1;
                }
                state
            }
            OwnedDataModelType::Schema => hash_update(state, &[0xE5]),
            OwnedDataModelType::Array { item, count } => {
                let mut state = hash_update(state, &[0x2B]);
                let mut idx = 0;

                // TODO: Do we want to hash the len or something? I'm not
                // sure how to handle this for different sizes of `usize`.
                while idx < *count {
                    state = hash_sdm_type_owned(state, item);
                    idx += 1;
                }
                state
            }
        }
    }

    const fn hash_struct(state: u64, _name: &str, data: &OwnedData) -> u64 {
        // NOTE: We do *not* hash the name of the type in hashv2. This
        // is to allow "safe" type punning, e.g. treating `Vec<u8>` and
        // `&[u8]` as compatible types, when talking between std and no-std
        // targets
        //
        // let state = hash_update(state, name.as_bytes());
        match data {
            OwnedData::Unit => hash_update(state, &[0xBF]),
            OwnedData::Newtype(dmt) => {
                let state = hash_update(state, &[0x9D]);
                hash_sdm_type_owned(state, dmt)
            }
            OwnedData::Tuple(dmts) => {
                let mut state = hash_update(state, &[0x05]);
                let mut idx = 0;
                while idx < dmts.len() {
                    state = hash_sdm_type_owned(state, &dmts[idx]);
                    idx += 1;
                }
                state
            }
            OwnedData::Struct(nfs) => {
                let mut state = hash_update(state, &[0x7F]);
                let mut idx = 0;
                while idx < nfs.len() {
                    state = hash_named_field(state, &nfs[idx]);
                    idx += 1;
                }
                state
            }
        }
    }

    const fn hash_variant(state: u64, nt: &OwnedVariant) -> u64 {
        let state = hash_update(state, nt.name.as_bytes());
        match &nt.data {
            OwnedData::Unit => hash_update(state, &[0xB5]),
            OwnedData::Newtype(t) => {
                let state = hash_update(state, &[0xDF]);
                hash_sdm_type_owned(state, t)
            }
            OwnedData::Tuple(ts) => {
                let mut state = hash_update(state, &[0xC7]);
                let mut idx = 0;
                while idx < ts.len() {
                    state = hash_sdm_type_owned(state, &ts[idx]);
                    idx += 1;
                }
                state
            }
            OwnedData::Struct(fields) => {
                let mut state = hash_update(state, &[0x67]);
                let mut idx = 0;
                while idx < fields.len() {
                    state = hash_named_field(state, &fields[idx]);
                    idx += 1;
                }
                state
            }
        }
    }

    const fn hash_named_field(state: u64, nt: &OwnedNamedField) -> u64 {
        let state = hash_update(state, nt.name.as_bytes());
        hash_sdm_type_owned(state, &nt.ty)
    }
}

#[cfg(test)]
mod test {
    use postcard_derive_ng::Schema;

    use super::fnv1a64::hash_ty_path;

    #[test]
    fn hash_stability() {
        #![allow(dead_code)]

        #[derive(Schema)]
        #[postcard(crate = crate)]
        struct Foo {
            a: u32,
            b: String,
        }

        #[derive(Schema)]
        #[postcard(crate = crate)]
        enum Bar {
            A,
            B(Foo),
        }

        assert_eq!(
            hash_ty_path::<Bar>("test_path"),
            [139, 128, 52, 27, 107, 8, 218, 98]
        );
    }

    #[test]
    fn type_punning_good() {
        let hash_1 = hash_ty_path::<Vec<u8>>("test_path");
        let hash_2 = hash_ty_path::<&[u8]>("test_path");
        let hash_3 = hash_ty_path::<Vec<u16>>("test_path");
        let hash_4 = hash_ty_path::<&[u16]>("test_path");
        let hash_5 = hash_ty_path::<Vec<u8>>("test_patt");
        let hash_6 = hash_ty_path::<&[u8]>("test_patt");
        assert_eq!(hash_1, hash_2);
        assert_eq!(hash_3, hash_4);
        assert_ne!(hash_1, hash_3);
        assert_ne!(hash_2, hash_4);
        assert_ne!(hash_1, hash_5);
        assert_ne!(hash_2, hash_6);
    }
}
