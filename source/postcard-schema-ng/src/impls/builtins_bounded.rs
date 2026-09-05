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

impl<'a, const N: usize> From<BoundedStr<'a, N>> for &'a str {
    fn from(value: BoundedStr<'a, N>) -> Self {
        value.inner
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
pub struct BoundedByteSlice<'a, const N: usize> {
    inner: &'a [u8],
}

impl<'a, const N: usize> core::fmt::Debug for BoundedByteSlice<'a, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, const N: usize> From<BoundedByteSlice<'a, N>> for &'a [u8] {
    fn from(value: BoundedByteSlice<'a, N>) -> Self {
        value.inner
    }
}

impl<'a, const N: usize> Deref for BoundedByteSlice<'a, N> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, const N: usize> AsRef<[u8]> for BoundedByteSlice<'a, N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.inner
    }
}

impl<'a, const N: usize> TryFrom<&'a [u8]> for BoundedByteSlice<'a, N> {
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

impl<'a, const N: usize, const M: usize> TryFrom<&'a [u8; M]> for BoundedByteSlice<'a, N> {
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

impl<'a, const N: usize> Serialize for BoundedByteSlice<'a, N> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, const N: usize> Deserialize<'de> for BoundedByteSlice<'de, N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor<const N: usize>;
        impl<'de, const N: usize> Visitor<'de> for BytesVisitor<N> {
            type Value = BoundedByteSlice<'de, N>;

            #[inline]
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(formatter, "bytes with fewer than {N} items")
            }

            #[inline]
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(b) = BoundedByteSlice::<'de, N>::try_from(v) {
                    Ok(b)
                } else {
                    Err(E::invalid_length(v.len(), &self))
                }
            }
        }

        deserializer.deserialize_bytes(BytesVisitor::<N>)
    }
}

impl<'a, const N: usize> Schema for BoundedByteSlice<'a, N> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq {
        item: &DataModelType::U8,
        bounds: Some(N),
    };
}

#[cfg(feature = "use-std")]
pub(crate) mod std {
    //
    // STRING
    //

    use core::ops::Deref;

    use serde::{de::Visitor, Deserialize, Serialize};

    use crate::{bounded::TooLong, schema::DataModelType, Schema};

    /// Bounded version of `String`
    #[derive(PartialEq, Clone)]
    pub struct BoundedString<const N: usize> {
        inner: String,
    }

    impl<const N: usize> core::fmt::Debug for BoundedString<N> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.inner.fmt(f)
        }
    }

    impl<const N: usize> core::fmt::Display for BoundedString<N> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.inner.fmt(f)
        }
    }

    impl<const N: usize> From<BoundedString<N>> for String {
        fn from(value: BoundedString<N>) -> Self {
            value.inner
        }
    }

    impl<const N: usize> Deref for BoundedString<N> {
        type Target = str;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    // We don't implement DerefMut because we don't want people extending
    // the string.

    impl<const N: usize> AsRef<str> for BoundedString<N> {
        #[inline]
        fn as_ref(&self) -> &str {
            &self.inner
        }
    }

    impl<const N: usize> TryFrom<&str> for BoundedString<N> {
        type Error = TooLong;

        #[inline]
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            if value.len() > N {
                Err(TooLong)
            } else {
                Ok(Self {
                    inner: value.to_string(),
                })
            }
        }
    }

    impl<const N: usize> TryFrom<String> for BoundedString<N> {
        type Error = TooLong;

        #[inline]
        fn try_from(value: String) -> Result<Self, Self::Error> {
            if value.len() > N {
                Err(TooLong)
            } else {
                Ok(Self { inner: value })
            }
        }
    }

    impl<const N: usize> Serialize for BoundedString<N> {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de, const N: usize> Deserialize<'de> for BoundedString<N> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct StrVisitor<const N: usize>;
            impl<'de, const N: usize> Visitor<'de> for StrVisitor<N> {
                type Value = BoundedString<N>;

                #[inline]
                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    write!(formatter, "a string with fewer than {N} chars")
                }

                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    let len = v.len();
                    if let Ok(b) = BoundedString::<N>::try_from(v) {
                        Ok(b)
                    } else {
                        Err(E::invalid_length(len, &self))
                    }
                }

                #[inline]
                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    if let Ok(b) = BoundedString::<N>::try_from(v) {
                        Ok(b)
                    } else {
                        Err(E::invalid_length(v.len(), &self))
                    }
                }
            }

            deserializer.deserialize_str(StrVisitor::<N>)
        }
    }

    impl<const N: usize> Schema for BoundedString<N> {
        const SCHEMA: &'static DataModelType = &DataModelType::String { bounds: Some(N) };
    }

    //
    // BYTES
    //
    /// Bounded version of `Box<[u8]>`
    #[derive(PartialEq, Clone)]
    pub struct BoundedBytes<const N: usize> {
        inner: Box<[u8]>,
    }

    impl<'a, const N: usize> core::fmt::Debug for BoundedBytes<N> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.inner.fmt(f)
        }
    }

    impl<const N: usize> Deref for BoundedBytes<N> {
        type Target = [u8];

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<const N: usize> AsRef<[u8]> for BoundedBytes<N> {
        #[inline]
        fn as_ref(&self) -> &[u8] {
            &self.inner
        }
    }

    impl<const N: usize> TryFrom<&[u8]> for BoundedBytes<N> {
        type Error = TooLong;

        #[inline]
        fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            if value.len() > N {
                Err(TooLong)
            } else {
                Ok(Self {
                    inner: value.into(),
                })
            }
        }
    }

    impl<const N: usize, const M: usize> TryFrom<&[u8; M]> for BoundedBytes<N> {
        type Error = TooLong;

        #[inline]
        fn try_from(value: &[u8; M]) -> Result<Self, Self::Error> {
            if M > N {
                Err(TooLong)
            } else {
                Ok(Self {
                    inner: value.as_slice().into(),
                })
            }
        }
    }

    impl<const N: usize> Serialize for BoundedBytes<N> {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de, const N: usize> Deserialize<'de> for BoundedBytes<N> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct BytesVisitor<const N: usize>;
            impl<'de, const N: usize> Visitor<'de> for BytesVisitor<N> {
                type Value = BoundedBytes<N>;

                #[inline]
                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    write!(formatter, "bytes with fewer than {N} items")
                }

                #[inline]
                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    if let Ok(b) = BoundedBytes::<N>::try_from(v) {
                        Ok(b)
                    } else {
                        Err(E::invalid_length(v.len(), &self))
                    }
                }
            }

            deserializer.deserialize_bytes(BytesVisitor::<N>)
        }
    }

    impl<const N: usize> Schema for BoundedBytes<N> {
        const SCHEMA: &'static DataModelType = &DataModelType::Seq {
            item: &DataModelType::U8,
            bounds: Some(N),
        };
    }

    #[cfg(test)]
    mod test {
        use postcard::Error::SerdeDeCustom;

        use crate::bounded::{BoundedByteSlice, BoundedStr};

        #[test]
        fn bounded_str() {
            use super::*;
            assert_eq!(BoundedString::<8>::SCHEMA.max_size(), Some(9));
            assert_eq!(BoundedString::<127>::SCHEMA.max_size(), Some(128));
            assert_eq!(BoundedString::<128>::SCHEMA.max_size(), Some(130));

            let b: BoundedString<5> = "hello".try_into().unwrap();
            let x = postcard::to_stdvec(&b).unwrap();
            let base = postcard::to_stdvec("hello").unwrap();
            assert_eq!(x, base);
            assert!(BoundedString::<4>::try_from("hello").is_err());

            let c = postcard::from_bytes::<BoundedString<5>>(&x).unwrap();
            assert_eq!(b, c);
            let c2 = postcard::from_bytes::<BoundedStr<'_, 5>>(&x).unwrap();
            assert_eq!(b.deref(), c2.deref());

            let d = postcard::from_bytes::<BoundedString<4>>(&x);
            assert_eq!(d, Err(SerdeDeCustom));

            assert_eq!(
                <BoundedString<5>>::SCHEMA,
                &DataModelType::String { bounds: Some(5) }
            );
            assert_eq!(<BoundedString<5>>::SCHEMA, <BoundedStr<'_, 5>>::SCHEMA);
        }

        #[test]
        fn bounded_bytes() {
            use super::*;
            assert_eq!(BoundedBytes::<8>::SCHEMA.max_size(), Some(9));
            assert_eq!(BoundedBytes::<127>::SCHEMA.max_size(), Some(128));
            assert_eq!(BoundedBytes::<128>::SCHEMA.max_size(), Some(130));

            let _b: BoundedBytes<6> = b"hello".try_into().unwrap();
            let b: BoundedBytes<5> = b"hello".try_into().unwrap();
            assert_eq!(b.get(..3), Some(b"hel".as_slice()));
            let x = postcard::to_stdvec(&b).unwrap();
            let base = postcard::to_stdvec(b"hello".as_slice()).unwrap();
            assert_eq!(x, base);
            assert!(BoundedBytes::<4>::try_from(b"hello").is_err());

            let c = postcard::from_bytes::<BoundedBytes<5>>(&x).unwrap();
            assert_eq!(b, c);
            let c2 = postcard::from_bytes::<BoundedByteSlice<'_, 5>>(&x).unwrap();
            assert_eq!(b.deref(), c2.deref());

            let d = postcard::from_bytes::<BoundedBytes<4>>(&x);
            assert_eq!(d, Err(SerdeDeCustom));

            assert_eq!(
                <BoundedBytes<5>>::SCHEMA,
                &DataModelType::Seq {
                    item: &DataModelType::U8,
                    bounds: Some(5),
                }
            );
            assert_eq!(<BoundedBytes<5>>::SCHEMA, <BoundedByteSlice<'_, 5>>::SCHEMA);
        }
    }
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
        assert_eq!(BoundedByteSlice::<8>::SCHEMA.max_size(), Some(9));
        assert_eq!(BoundedByteSlice::<127>::SCHEMA.max_size(), Some(128));
        assert_eq!(BoundedByteSlice::<128>::SCHEMA.max_size(), Some(130));

        let _b: BoundedByteSlice<'_, 6> = b"hello".try_into().unwrap();
        let b: BoundedByteSlice<'_, 5> = b"hello".try_into().unwrap();
        assert_eq!(b.get(..3), Some(b"hel".as_slice()));
        let x = postcard::to_stdvec(&b).unwrap();
        let base = postcard::to_stdvec(b"hello".as_slice()).unwrap();
        assert_eq!(x, base);
        assert!(BoundedByteSlice::<4>::try_from(b"hello").is_err());

        let c = postcard::from_bytes::<BoundedByteSlice<5>>(&x).unwrap();
        assert_eq!(b, c);

        let d = postcard::from_bytes::<BoundedByteSlice<4>>(&x);
        assert_eq!(d, Err(SerdeDeCustom));

        assert_eq!(
            <BoundedByteSlice<'_, 5>>::SCHEMA,
            &DataModelType::Seq {
                item: &DataModelType::U8,
                bounds: Some(5),
            }
        );
    }
}
