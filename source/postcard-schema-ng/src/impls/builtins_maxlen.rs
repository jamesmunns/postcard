//! MaxLen collections
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

/// MaxLen version of `&str`
#[derive(PartialEq, Clone, Copy)]
pub struct MaxLenStr<'a, const N: usize> {
    inner: &'a str,
}

impl<'a, const N: usize> core::fmt::Debug for MaxLenStr<'a, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, const N: usize> core::fmt::Display for MaxLenStr<'a, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, const N: usize> From<MaxLenStr<'a, N>> for &'a str {
    fn from(value: MaxLenStr<'a, N>) -> Self {
        value.inner
    }
}

impl<'a, const N: usize> Deref for MaxLenStr<'a, N> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, const N: usize> AsRef<str> for MaxLenStr<'a, N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.inner
    }
}

impl<'a, const N: usize> TryFrom<&'a str> for MaxLenStr<'a, N> {
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

impl<'a, const N: usize> Serialize for MaxLenStr<'a, N> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.inner)
    }
}

impl<'de, const N: usize> Deserialize<'de> for MaxLenStr<'de, N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrVisitor<const N: usize>;
        impl<'de, const N: usize> Visitor<'de> for StrVisitor<N> {
            type Value = MaxLenStr<'de, N>;

            #[inline]
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(formatter, "a string with fewer than {N} chars")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(b) = MaxLenStr::<'de, N>::try_from(v) {
                    Ok(b)
                } else {
                    Err(E::invalid_length(v.len(), &self))
                }
            }
        }

        deserializer.deserialize_str(StrVisitor::<N>)
    }
}

impl<'a, const N: usize> Schema for MaxLenStr<'a, N> {
    const SCHEMA: &'static DataModelType = &DataModelType::String { max_len: Some(N) };
}

//
// BYTES
//
/// MaxLen version of `&[u8]`
#[derive(PartialEq, Clone, Copy)]
pub struct MaxLenByteSlice<'a, const N: usize> {
    inner: &'a [u8],
}

impl<'a, const N: usize> core::fmt::Debug for MaxLenByteSlice<'a, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, const N: usize> From<MaxLenByteSlice<'a, N>> for &'a [u8] {
    fn from(value: MaxLenByteSlice<'a, N>) -> Self {
        value.inner
    }
}

impl<'a, const N: usize> Deref for MaxLenByteSlice<'a, N> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, const N: usize> AsRef<[u8]> for MaxLenByteSlice<'a, N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.inner
    }
}

impl<'a, const N: usize> TryFrom<&'a [u8]> for MaxLenByteSlice<'a, N> {
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

impl<'a, const N: usize, const M: usize> TryFrom<&'a [u8; M]> for MaxLenByteSlice<'a, N> {
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

impl<'a, const N: usize> Serialize for MaxLenByteSlice<'a, N> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.inner)
    }
}

impl<'de, const N: usize> Deserialize<'de> for MaxLenByteSlice<'de, N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor<const N: usize>;
        impl<'de, const N: usize> Visitor<'de> for BytesVisitor<N> {
            type Value = MaxLenByteSlice<'de, N>;

            #[inline]
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(formatter, "bytes with fewer than {N} items")
            }

            #[inline]
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(b) = MaxLenByteSlice::<'de, N>::try_from(v) {
                    Ok(b)
                } else {
                    Err(E::invalid_length(v.len(), &self))
                }
            }
        }

        deserializer.deserialize_bytes(BytesVisitor::<N>)
    }
}

impl<'a, const N: usize> Schema for MaxLenByteSlice<'a, N> {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq {
        element: &DataModelType::U8,
        max_len: Some(N),
    };
}

#[cfg(feature = "use-std")]
pub(crate) mod std {
    //
    // STRING
    //

    use core::ops::Deref;

    use serde::{de::Visitor, Deserialize, Serialize};

    use crate::{max_len::TooLong, schema::DataModelType, Schema};

    /// MaxLen version of `String`
    #[derive(PartialEq, Clone)]
    pub struct MaxLenString<const N: usize> {
        inner: String,
    }

    impl<const N: usize> core::fmt::Debug for MaxLenString<N> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.inner.fmt(f)
        }
    }

    impl<const N: usize> core::fmt::Display for MaxLenString<N> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.inner.fmt(f)
        }
    }

    impl<const N: usize> From<MaxLenString<N>> for String {
        fn from(value: MaxLenString<N>) -> Self {
            value.inner
        }
    }

    impl<const N: usize> Deref for MaxLenString<N> {
        type Target = str;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    // We don't implement DerefMut because we don't want people extending
    // the string.

    impl<const N: usize> AsRef<str> for MaxLenString<N> {
        #[inline]
        fn as_ref(&self) -> &str {
            &self.inner
        }
    }

    impl<const N: usize> TryFrom<&str> for MaxLenString<N> {
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

    impl<const N: usize> TryFrom<String> for MaxLenString<N> {
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

    impl<const N: usize> Serialize for MaxLenString<N> {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.inner)
        }
    }

    impl<'de, const N: usize> Deserialize<'de> for MaxLenString<N> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct StrVisitor<const N: usize>;
            impl<'de, const N: usize> Visitor<'de> for StrVisitor<N> {
                type Value = MaxLenString<N>;

                #[inline]
                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    write!(formatter, "a string with fewer than {N} chars")
                }

                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    let len = v.len();
                    if let Ok(b) = MaxLenString::<N>::try_from(v) {
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
                    if let Ok(b) = MaxLenString::<N>::try_from(v) {
                        Ok(b)
                    } else {
                        Err(E::invalid_length(v.len(), &self))
                    }
                }
            }

            deserializer.deserialize_str(StrVisitor::<N>)
        }
    }

    impl<const N: usize> Schema for MaxLenString<N> {
        const SCHEMA: &'static DataModelType = &DataModelType::String { max_len: Some(N) };
    }

    //
    // BYTES
    //
    /// MaxLen version of `Box<[u8]>`
    #[derive(PartialEq, Clone)]
    pub struct MaxLenBytes<const N: usize> {
        inner: Box<[u8]>,
    }

    impl<const N: usize> core::fmt::Debug for MaxLenBytes<N> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.inner.fmt(f)
        }
    }

    impl<const N: usize> Deref for MaxLenBytes<N> {
        type Target = [u8];

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<const N: usize> AsRef<[u8]> for MaxLenBytes<N> {
        #[inline]
        fn as_ref(&self) -> &[u8] {
            &self.inner
        }
    }

    impl<const N: usize> TryFrom<&[u8]> for MaxLenBytes<N> {
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

    impl<const N: usize, const M: usize> TryFrom<&[u8; M]> for MaxLenBytes<N> {
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

    impl<const N: usize> Serialize for MaxLenBytes<N> {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_bytes(&self.inner)
        }
    }

    impl<'de, const N: usize> Deserialize<'de> for MaxLenBytes<N> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct BytesVisitor<const N: usize>;
            impl<'de, const N: usize> Visitor<'de> for BytesVisitor<N> {
                type Value = MaxLenBytes<N>;

                #[inline]
                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    write!(formatter, "bytes with fewer than {N} items")
                }

                #[inline]
                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    if let Ok(b) = MaxLenBytes::<N>::try_from(v) {
                        Ok(b)
                    } else {
                        Err(E::invalid_length(v.len(), &self))
                    }
                }
            }

            deserializer.deserialize_bytes(BytesVisitor::<N>)
        }
    }

    impl<const N: usize> Schema for MaxLenBytes<N> {
        const SCHEMA: &'static DataModelType = &DataModelType::Seq {
            element: &DataModelType::U8,
            max_len: Some(N),
        };
    }

    #[cfg(test)]
    mod test {
        use postcard::Error::SerdeDeCustom;

        use crate::max_len::{MaxLenByteSlice, MaxLenStr};

        #[test]
        fn bounded_str() {
            use super::*;
            assert_eq!(MaxLenString::<8>::SCHEMA.max_size(), Some(9));
            assert_eq!(MaxLenString::<127>::SCHEMA.max_size(), Some(128));
            assert_eq!(MaxLenString::<128>::SCHEMA.max_size(), Some(130));

            let b: MaxLenString<5> = "hello".try_into().unwrap();
            let x = postcard::to_stdvec(&b).unwrap();
            let base = postcard::to_stdvec("hello").unwrap();
            assert_eq!(x, base);
            assert!(MaxLenString::<4>::try_from("hello").is_err());

            let c = postcard::from_bytes::<MaxLenString<5>>(&x).unwrap();
            assert_eq!(b, c);
            let c2 = postcard::from_bytes::<MaxLenStr<'_, 5>>(&x).unwrap();
            assert_eq!(b.deref(), c2.deref());

            let d = postcard::from_bytes::<MaxLenString<4>>(&x);
            assert_eq!(d, Err(SerdeDeCustom));

            assert_eq!(
                <MaxLenString<5>>::SCHEMA,
                &DataModelType::String { max_len: Some(5) }
            );
            assert_eq!(<MaxLenString<5>>::SCHEMA, <MaxLenStr<'_, 5>>::SCHEMA);
        }

        #[test]
        fn bounded_bytes() {
            use super::*;
            assert_eq!(MaxLenBytes::<8>::SCHEMA.max_size(), Some(9));
            assert_eq!(MaxLenBytes::<127>::SCHEMA.max_size(), Some(128));
            assert_eq!(MaxLenBytes::<128>::SCHEMA.max_size(), Some(130));

            let _b: MaxLenBytes<6> = b"hello".try_into().unwrap();
            let b: MaxLenBytes<5> = b"hello".try_into().unwrap();
            assert_eq!(b.get(..3), Some(b"hel".as_slice()));
            let x = postcard::to_stdvec(&b).unwrap();
            let base = postcard::to_stdvec(b"hello".as_slice()).unwrap();
            assert_eq!(x, base);
            assert!(MaxLenBytes::<4>::try_from(b"hello").is_err());

            let c = postcard::from_bytes::<MaxLenBytes<5>>(&x).unwrap();
            assert_eq!(b, c);
            let c2 = postcard::from_bytes::<MaxLenByteSlice<'_, 5>>(&x).unwrap();
            assert_eq!(b.deref(), c2.deref());

            let d = postcard::from_bytes::<MaxLenBytes<4>>(&x);
            assert_eq!(d, Err(SerdeDeCustom));

            assert_eq!(
                <MaxLenBytes<5>>::SCHEMA,
                &DataModelType::Seq {
                    element: &DataModelType::U8,
                    max_len: Some(5),
                }
            );
            assert_eq!(<MaxLenBytes<5>>::SCHEMA, <MaxLenByteSlice<'_, 5>>::SCHEMA);
        }
    }
}

#[cfg(test)]
mod test {
    use postcard::Error::SerdeDeCustom;

    #[test]
    fn bounded_str() {
        use super::*;
        assert_eq!(MaxLenStr::<8>::SCHEMA.max_size(), Some(9));
        assert_eq!(MaxLenStr::<127>::SCHEMA.max_size(), Some(128));
        assert_eq!(MaxLenStr::<128>::SCHEMA.max_size(), Some(130));

        let b: MaxLenStr<'_, 5> = "hello".try_into().unwrap();
        let x = postcard::to_stdvec(&b).unwrap();
        let base = postcard::to_stdvec("hello").unwrap();
        assert_eq!(x, base);
        assert!(MaxLenStr::<4>::try_from("hello").is_err());

        let c = postcard::from_bytes::<MaxLenStr<5>>(&x).unwrap();
        assert_eq!(b, c);

        let d = postcard::from_bytes::<MaxLenStr<4>>(&x);
        assert_eq!(d, Err(SerdeDeCustom));

        assert_eq!(
            <MaxLenStr<'_, 5>>::SCHEMA,
            &DataModelType::String { max_len: Some(5) }
        );
    }

    #[test]
    fn bounded_bytes() {
        use super::*;
        assert_eq!(MaxLenByteSlice::<8>::SCHEMA.max_size(), Some(9));
        assert_eq!(MaxLenByteSlice::<127>::SCHEMA.max_size(), Some(128));
        assert_eq!(MaxLenByteSlice::<128>::SCHEMA.max_size(), Some(130));

        let _b: MaxLenByteSlice<'_, 6> = b"hello".try_into().unwrap();
        let b: MaxLenByteSlice<'_, 5> = b"hello".try_into().unwrap();
        assert_eq!(b.get(..3), Some(b"hel".as_slice()));
        let x = postcard::to_stdvec(&b).unwrap();
        let base = postcard::to_stdvec(b"hello".as_slice()).unwrap();
        assert_eq!(x, base);
        assert!(MaxLenByteSlice::<4>::try_from(b"hello").is_err());

        let c = postcard::from_bytes::<MaxLenByteSlice<5>>(&x).unwrap();
        assert_eq!(b, c);

        let d = postcard::from_bytes::<MaxLenByteSlice<4>>(&x);
        assert_eq!(d, Err(SerdeDeCustom));

        assert_eq!(
            <MaxLenByteSlice<'_, 5>>::SCHEMA,
            &DataModelType::Seq {
                element: &DataModelType::U8,
                max_len: Some(5),
            }
        );
    }
}
