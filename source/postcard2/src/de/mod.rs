use serde_core::Deserialize;

pub(crate) mod deserializer;
pub mod flavors;

use deserializer::{Deserializer, DeserializerError};

/// Deserialize a message of type `T` from a byte slice. The unused portion (if any)
/// of the byte slice is not returned.
pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T, DeserializerError>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

/// Deserialize a message of type `T` from a byte slice. The unused portion (if any)
/// of the byte slice is returned for further usage
pub fn take_from_bytes<'a, T>(s: &'a [u8]) -> Result<(T, &'a [u8]), DeserializerError>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok((t, deserializer.finalize()?))
}

/// Deserialize a message of type `T` from a [`std::io::Read`].
#[cfg(feature = "std")]
pub fn from_io<'a, T, R>(
    val: (R, &'a mut [u8]),
) -> Result<(T, (R, &'a mut [u8])), DeserializerError>
where
    T: Deserialize<'a>,
    R: std::io::Read + 'a,
{
    let flavor = flavors::io::io::IOReader::new(val.0, val.1);
    let mut deserializer = Deserializer::from_flavor(flavor);
    let t = T::deserialize(&mut deserializer)?;
    Ok((t, deserializer.finalize()?))
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg(test)]
mod test_alloc {
    extern crate alloc;

    use super::*;

    use alloc::vec;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct ZSTStruct;

    #[test]
    fn zst_vec() {
        assert_eq!(from_bytes(&[3]), Ok(vec![ZSTStruct, ZSTStruct, ZSTStruct]));

        assert_eq!(
            from_bytes(&[4]),
            Ok(vec![ZSTStruct, ZSTStruct, ZSTStruct, ZSTStruct])
        );
    }

    #[test]
    fn vec() {
        assert_eq!(
            from_bytes::<Vec<u8>>(&[8, 255, 255, 255, 0, 0, 0, 0, 0]),
            Ok(vec![255, 255, 255, 0, 0, 0, 0, 0])
        );

        // This won't actually prove anything since tests will likely always be
        // run on devices with larger amounts of memory, but it can't hurt.
        assert_eq!(
            from_bytes::<Vec<u8>>(&[(1 << 7) | 8, 255, 255, 255, 0, 0, 0, 0, 0]),
            Err(crate::de::deserializer::DeserializerError::Flavor(
                crate::de_flavors::DeFlavorError::UnexpectedEnd
            ))
        );
    }
}
