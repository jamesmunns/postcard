//! Bounded collections
//!
//! These items have schemas that include bounded information.

use crate::{schema::DataModelType, Schema};
use core::ops::Deref;
use serde::{de::Visitor, Deserialize, Serialize};

/// Provided item exceeded the bounds of this type
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TooLong;

//
// STR
//

/// Bounded version of `&str`
#[derive(PartialEq, Clone, Copy)]
pub struct BoundedStr<'a, const N: usize> {
    inner: &'a str,
}

impl<'a, const N: usize> core::fmt::Debug for BoundedStr<'a, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, const N: usize> core::fmt::Display for BoundedStr<'a, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, const N: usize> Deref for BoundedStr<'a, N> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, const N: usize> AsRef<str> for BoundedStr<'a, N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.inner
    }
}

impl<'a, const N: usize> TryFrom<&'a str> for BoundedStr<'a, N> {
    type Error = TooLong;

    #[inline]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.len() > N {
            Err(TooLong)
        } else {
            Ok(Self { inner: value })
        }
    }
}

impl<'a, const N: usize> Serialize for BoundedStr<'a, N> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, const N: usize> Deserialize<'de> for BoundedStr<'de, N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrVisitor<const N: usize>;
        impl<'de, const N: usize> Visitor<'de> for StrVisitor<N> {
            type Value = BoundedStr<'de, N>;

            #[inline]
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(formatter, "a string with fewer than {N} chars")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(b) = BoundedStr::<'de, N>::try_from(v) {
                    Ok(b)
                } else {
                    Err(E::invalid_length(v.len(), &self))
                }
            }
        }

        deserializer.deserialize_str(StrVisitor::<N>)
    }
}

impl<'a, const N: usize> Schema for BoundedStr<'a, N> {
    const SCHEMA: &'static DataModelType = &DataModelType::String { bounds: Some(N) };
}

//
// BYTES
//
/// Bounded version of `&[u8]`
#[derive(PartialEq, Clone, Copy)]
pub struct BoundedBytes<'a, const N: usize> {
    inner: &'a [u8],
}

impl<'a, const N: usize> core::fmt::Debug for BoundedBytes<'a, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, const N: usize> Deref for BoundedBytes<'a, N> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, const N: usize> AsRef<[u8]> for BoundedBytes<'a, N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.inner
    }
}

impl<'a, const N: usize> TryFrom<&'a [u8]> for BoundedBytes<'a, N> {
    type Error = TooLong;

    #[inline]
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() > N {
            Err(TooLong)
        } else {
            Ok(Self { inner: value })
        }
    }
}

impl<'a, const N: usize, const M: usize> TryFrom<&'a [u8; M]> for BoundedBytes<'a, N> {
    type Error = TooLong;

    #[inline]
    fn try_from(value: &'a [u8; M]) -> Result<Self, Self::Error> {
        if M > N {
            Err(TooLong)
        } else {
            Ok(Self {
                inner: value.as_slice(),
            })
        }
    }
}

impl<'a, const N: usize> Serialize for BoundedBytes<'a, N> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, const N: usize> Deserialize<'de> for BoundedBytes<'de, N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor<const N: usize>;
        impl<'de, const N: usize> Visitor<'de> for BytesVisitor<N> {
            type Value = BoundedBytes<'de, N>;

            #[inline]
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(formatter, "bytes with fewer than {N} items")
            }

            #[inline]
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(b) = BoundedBytes::<'de, N>::try_from(v) {
                    Ok(b)
                } else {
                    Err(E::invalid_length(v.len(), &self))
                }
            }
        }

        deserializer.deserialize_bytes(BytesVisitor::<N>)
    }
}

impl<'a, const N: usize> Schema for BoundedBytes<'a, N> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq {
        item: &DataModelType::U8,
        bounds: Some(N),
    };
}

#[cfg(test)]
mod test {
    use postcard::Error::SerdeDeCustom;

    #[test]
    fn bounded_str() {
        use super::*;
        assert_eq!(BoundedStr::<8>::SCHEMA.max_size(), Some(9));
        assert_eq!(BoundedStr::<127>::SCHEMA.max_size(), Some(128));
        assert_eq!(BoundedStr::<128>::SCHEMA.max_size(), Some(130));

        let b: BoundedStr<'_, 5> = "hello".try_into().unwrap();
        let x = postcard::to_stdvec(&b).unwrap();
        let base = postcard::to_stdvec("hello").unwrap();
        assert_eq!(x, base);
        assert!(BoundedStr::<4>::try_from("hello").is_err());

        let c = postcard::from_bytes::<BoundedStr<5>>(&x).unwrap();
        assert_eq!(b, c);

        let d = postcard::from_bytes::<BoundedStr<4>>(&x);
        assert_eq!(d, Err(SerdeDeCustom));

        assert_eq!(
            <BoundedStr<'_, 5>>::SCHEMA,
            &DataModelType::String { bounds: Some(5) }
        );
    }

    #[test]
    fn bounded_bytes() {
        use super::*;
        assert_eq!(BoundedBytes::<8>::SCHEMA.max_size(), Some(9));
        assert_eq!(BoundedBytes::<127>::SCHEMA.max_size(), Some(128));
        assert_eq!(BoundedBytes::<128>::SCHEMA.max_size(), Some(130));

        let _b: BoundedBytes<'_, 6> = b"hello".try_into().unwrap();
        let b: BoundedBytes<'_, 5> = b"hello".try_into().unwrap();
        assert_eq!(b.get(..3), Some(b"hel".as_slice()));
        let x = postcard::to_stdvec(&b).unwrap();
        let base = postcard::to_stdvec(b"hello".as_slice()).unwrap();
        assert_eq!(x, base);
        assert!(BoundedBytes::<4>::try_from(b"hello").is_err());

        let c = postcard::from_bytes::<BoundedBytes<5>>(&x).unwrap();
        assert_eq!(b, c);

        let d = postcard::from_bytes::<BoundedBytes<4>>(&x);
        assert_eq!(d, Err(SerdeDeCustom));

        assert_eq!(
            <BoundedBytes<'_, 5>>::SCHEMA,
            &DataModelType::Seq {
                item: &DataModelType::U8,
                bounds: Some(5),
            }
        );
    }
}
