//! # Fixed Size Integers
//!
//! In some cases, the use of variably length encoded data may not be
//! preferrable. These wrappers "opt out" of variable length encoding.
//!
//! A wrapper is explicitly not provided for `usize` or `isize`, as
//! these types would not be portable between systems of different
//! pointer widths.
//!
//! Although all data in Postcard is typically encoded in little-endian
//! order, these wrappers provide a choice to the user to encode the data
//! in either little or big endian form, which may be useful for zero-copy
//! applications.
use serde::{Deserialize, Serialize, Serializer};

/// Type wrapper that disables varint serialization/deserialization
/// for the wrapped integer. The contained integer will always be serialized
/// in the same way as a fixed size array, in **Little Endian** order on
/// the wire.
///
/// The type remains as a standard, "native endian", integer while in-memory.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[repr(transparent)]
pub struct FixintLE<T: sealed::Fixed>(pub T);

/// Type wrapper that disables varint serialization/deserialization
/// for the wrapped integer. The contained integer will always be serialized
/// in the same way as a fixed size array, in **Big Endian** order on
/// the wire.
///
/// The type remains as a standard, "native endian", integer while in-memory.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[repr(transparent)]
pub struct FixintBE<T: sealed::Fixed>(pub T);

mod sealed {
    pub trait Fixed {}
}

macro_rules! impl_fixint {
    ($( $int:ty ),*) => {
        $(
            // Sealed trait to avoid misuse
            impl sealed::Fixed for $int { }

            // Little Endian Wrappers

            impl From<$int> for FixintLE<$int> {
                #[inline]
                fn from(x: $int) -> Self {
                    Self(x)
                }
            }

            impl From<FixintLE<$int>> for $int {
                #[inline]
                fn from(x: FixintLE<$int>) -> $int {
                    x.0
                }
            }

            impl Serialize for FixintLE<$int> {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    self.0.to_le_bytes().serialize(serializer)
                }
            }

            impl<'de> Deserialize<'de> for FixintLE<$int> {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    <_ as Deserialize>::deserialize(deserializer)
                        .map(<$int>::from_le_bytes)
                        .map(Self)
                }
            }

            // Big Endian Wrappers

            impl From<$int> for FixintBE<$int> {
                #[inline]
                fn from(x: $int) -> Self {
                    Self(x)
                }
            }

            impl From<FixintBE<$int>> for $int {
                #[inline]
                fn from(x: FixintBE<$int>) -> $int {
                    x.0
                }
            }

            impl Serialize for FixintBE<$int> {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    self.0.to_be_bytes().serialize(serializer)
                }
            }

            impl<'de> Deserialize<'de> for FixintBE<$int> {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    <_ as Deserialize>::deserialize(deserializer)
                        .map(<$int>::from_be_bytes)
                        .map(Self)
                }
            }
        )*
    };
}

impl_fixint![i16, i32, i64, i128, u16, u32, u64, u128];
