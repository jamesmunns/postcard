use serde::ser::SerializeTupleStruct;
use serde::Serialize;
use serde::Serializer;

use core::convert::TryInto;
use core::fmt;
use serde::Deserializer;
use serde::Deserialize;
use serde::de;

/// Represents a fixed-size byte array for better performance with `postcard`.
///
/// This struct *only* works with `postcard` (de-)serialization.
#[repr(transparent)]
pub struct FixedSizeByteArray<const N: usize> {
    inner: FixedSizeByteArrayInner<N>,
}

impl<const N: usize> From<[u8; N]> for FixedSizeByteArray<N> {
    fn from(array: [u8; N]) -> FixedSizeByteArray<N> {
        FixedSizeByteArray {
            inner: FixedSizeByteArrayInner {
                array,
            },
        }
    }
}

impl<const N: usize> FixedSizeByteArray<N> {
    /// Extract the actual array.
    pub fn into_inner(self) -> [u8; N] {
        self.inner.array
    }
}

#[repr(transparent)]
struct FixedSizeByteArrayInner<const N: usize> {
    array: [u8; N],
}

pub static TOKEN: &str = "$postcard::private::FixedSizeByteArray";

impl<const N: usize> Serialize for FixedSizeByteArray<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple_struct(TOKEN, 1)?;
        s.serialize_field(&self.inner)?;
        s.end()
    }
}

impl<const N: usize> Serialize for FixedSizeByteArrayInner<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.array)
    }
}

impl<'de, const N: usize> Deserialize<'de> for FixedSizeByteArray<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<const N: usize>;

        impl<'de, const N: usize> de::Visitor<'de> for Visitor<N> {
            type Value = FixedSizeByteArray<N>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "byte array of length {}", N)
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let array: [u8; N] = match v.try_into() {
                    Ok(a) => a,
                    Err(_) => return Err(de::Error::invalid_length(v.len(), &self)),
                };
                Ok(FixedSizeByteArray::from(array))
            }
        }

        deserializer.deserialize_tuple_struct(TOKEN, N, Visitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;
    use super::FixedSizeByteArray;

    #[test]
    fn test_byte_array_serialize() {
        let empty = FixedSizeByteArray::from([]);
        let mut buf = [0; 32];
        let serialized = crate::to_slice(&empty, &mut buf).unwrap();
        assert_eq!(serialized, &[]);

        let single = FixedSizeByteArray::from([0x12]);
        let mut buf = [0; 32];
        let serialized = crate::to_slice(&single, &mut buf).unwrap();
        assert_eq!(serialized, &[0x12]);

        let five_bytes = FixedSizeByteArray::from([0x12, 0x34, 0x56, 0x78, 0x90]);
        let mut buf = [0; 32];
        let serialized = crate::to_slice(&five_bytes, &mut buf).unwrap();
        assert_eq!(serialized, &[0x12, 0x34, 0x56, 0x78, 0x90]);
    }

    #[test]
    fn test_byte_array_deserialize() {
        let deserialized: FixedSizeByteArray<0> = crate::from_bytes(&[]).unwrap();
        assert_eq!(deserialized.into_inner(), []);

        let deserialized: FixedSizeByteArray<0> = crate::from_bytes(&[0x12]).unwrap();
        assert_eq!(deserialized.into_inner(), []);

        let deserialized: FixedSizeByteArray<1> = crate::from_bytes(&[0x12]).unwrap();
        assert_eq!(deserialized.into_inner(), [0x12]);

        let deserialized: FixedSizeByteArray<5> = crate::from_bytes(&[0x12, 0x34, 0x56, 0x78, 0x90]).unwrap();
        assert_eq!(deserialized.into_inner(), [0x12, 0x34, 0x56, 0x78, 0x90]);
    }

    #[test]
    fn test_byte_array_deserialize_error() {
        let result: Result<FixedSizeByteArray<1>, _> = crate::from_bytes(&[]);
        assert_eq!(result.err().unwrap(), Error::DeserializeUnexpectedEnd);

        let result: Result<FixedSizeByteArray<8>, _> = crate::from_bytes(&[0x12]);
        assert_eq!(result.err().unwrap(), Error::DeserializeUnexpectedEnd);

        let result: Result<FixedSizeByteArray<8>, _> = crate::from_bytes(&[0x12, 0x34, 0x56, 0x78]);
        assert_eq!(result.err().unwrap(), Error::DeserializeUnexpectedEnd);
    }
}
