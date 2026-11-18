use core::fmt::{self, Display, Formatter};
use serde::de::{
    Deserialize, DeserializeSeed, Deserializer, EnumAccess, Unexpected, VariantAccess as _, Visitor,
};
use serde::ser::{Serialize, Serializer};

/// This is the error type used by Postcard
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// This is a feature that postcard will never implement
    WontImplement,
    /// This is a feature that postcard intends to support, but does not yet
    NotYetImplemented,
    /// The serialize buffer is full
    SerializeBufferFull,
    /// The length of a sequence must be known
    SerializeSeqLengthUnknown,
    /// Hit the end of buffer, expected more data
    DeserializeUnexpectedEnd,
    /// Found a varint that didn't terminate. Is the usize too big for this platform?
    DeserializeBadVarint,
    /// Found a bool that wasn't 0 or 1
    DeserializeBadBool,
    /// Found an invalid unicode char
    DeserializeBadChar,
    /// Tried to parse invalid utf-8
    DeserializeBadUtf8,
    /// Found an Option discriminant that wasn't 0 or 1
    DeserializeBadOption,
    /// Found an enum discriminant that was > `u32::MAX`
    DeserializeBadEnum,
    /// The original data was not well encoded
    DeserializeBadEncoding,
    /// Bad CRC while deserializing
    DeserializeBadCrc,
    #[cfg(feature = "use-std")]
    /// Serde Serialization Error
    SerdeSerCustom(String),
    #[cfg(not(feature = "use-std"))]
    /// Serde Serialization Error
    SerdeSerCustom,
    #[cfg(feature = "use-std")]
    /// Serde Deserialization Error
    SerdeDeCustom(String),
    #[cfg(not(feature = "use-std"))]
    /// Serde Deserialization Error
    SerdeDeCustom,
    /// Error while processing `collect_str` during serialization
    CollectStrError,
}

/// Names used for the serialized representation of `Error` in human-readable
/// formats.
const VARIANT_NAMES: [&str; 16] = [
    "WontImplement",
    "NotYetImplemented",
    "SerializeBufferFull",
    "SerializeSeqLengthUnknown",
    "DeserializeUnexpectedEnd",
    "DeserializeBadVarint",
    "DeserializeBadBool",
    "DeserializeBadChar",
    "DeserializeBadUtf8",
    "DeserializeBadOption",
    "DeserializeBadEnum",
    "DeserializeBadEncoding",
    "DeserializeBadCrc",
    "SerdeSerCustom",
    "SerdeDeCustom",
    "CollectStrError",
];

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use Error::*;
        write!(
            f,
            "{}",
            match self {
                WontImplement => "This is a feature that PostCard will never implement",
                NotYetImplemented => {
                    "This is a feature that Postcard intends to support, but does not yet"
                }
                SerializeBufferFull => "The serialize buffer is full",
                SerializeSeqLengthUnknown => "The length of a sequence must be known",
                DeserializeUnexpectedEnd => "Hit the end of buffer, expected more data",
                DeserializeBadVarint => {
                    "Found a varint that didn't terminate. Is the usize too big for this platform?"
                }
                DeserializeBadBool => "Found a bool that wasn't 0 or 1",
                DeserializeBadChar => "Found an invalid unicode char",
                DeserializeBadUtf8 => "Tried to parse invalid utf-8",
                DeserializeBadOption => "Found an Option discriminant that wasn't 0 or 1",
                DeserializeBadEnum => "Found an enum discriminant that was > u32::max_value()",
                DeserializeBadEncoding => "The original data was not well encoded",
                DeserializeBadCrc => "Bad CRC while deserializing",
                #[cfg(feature = "use-std")]
                SerdeSerCustom(s) => s,
                #[cfg(feature = "use-std")]
                SerdeDeCustom(s) => s,
                #[cfg(not(feature = "use-std"))]
                SerdeSerCustom => "Serde Serialization Error",
                #[cfg(not(feature = "use-std"))]
                SerdeDeCustom => "Serde Deserialization Error",
                CollectStrError => "Error while processing `collect_str` during serialization",
            }
        )
    }
}

/// This is the Result type used by Postcard.
pub type Result<T> = ::core::result::Result<T, Error>;

#[cfg(feature = "use-std")]
impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        // We must convert the parameter immediately to a string, because we cannot guarantee
        // that T lives long enough to be used later.
        Error::SerdeSerCustom(msg.to_string())
    }
}

#[cfg(not(feature = "use-std"))]
impl serde::ser::Error for Error {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        Error::SerdeSerCustom
    }
}

#[cfg(feature = "use-std")]
impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::SerdeDeCustom(msg.to_string())
    }
}

#[cfg(not(feature = "use-std"))]
impl serde::de::Error for Error {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        Error::SerdeDeCustom
    }
}

impl serde::ser::StdError for Error {}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit_variant(
            "Error",
            self.clone() as u32,
            VARIANT_NAMES[self.clone() as usize],
        )
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ErrorVisitor;

        impl<'de> DeserializeSeed<'de> for ErrorVisitor {
            type Value = Error;

            fn deserialize<D>(self, deserializer: D) -> core::result::Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_identifier(self)
            }
        }

        impl<'de> Visitor<'de> for ErrorVisitor {
            type Value = Error;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("postcard Error")
            }

            fn visit_enum<D>(self, data: D) -> core::result::Result<Self::Value, D::Error>
            where
                D: EnumAccess<'de>,
            {
                let (variant, contents) = data.variant_seed(self)?;
                contents.unit_variant()?;
                Ok(variant)
            }

            fn visit_u64<E>(self, value: u64) -> core::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0 => Ok(Error::WontImplement),
                    1 => Ok(Error::NotYetImplemented),
                    2 => Ok(Error::SerializeBufferFull),
                    3 => Ok(Error::SerializeSeqLengthUnknown),
                    4 => Ok(Error::DeserializeUnexpectedEnd),
                    5 => Ok(Error::DeserializeBadVarint),
                    6 => Ok(Error::DeserializeBadBool),
                    7 => Ok(Error::DeserializeBadChar),
                    8 => Ok(Error::DeserializeBadUtf8),
                    9 => Ok(Error::DeserializeBadOption),
                    10 => Ok(Error::DeserializeBadEnum),
                    11 => Ok(Error::DeserializeBadEncoding),
                    12 => Ok(Error::DeserializeBadCrc),
                    13 => Ok(Error::SerdeSerCustom),
                    14 => Ok(Error::SerdeDeCustom),
                    15 => Ok(Error::CollectStrError),
                    _ => Err(E::invalid_value(
                        Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 16",
                    )),
                }
            }

            fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "WontImplement" => Ok(Error::WontImplement),
                    "NotYetImplemented" => Ok(Error::NotYetImplemented),
                    "SerializeBufferFull" => Ok(Error::SerializeBufferFull),
                    "SerializeSeqLengthUnknown" => Ok(Error::SerializeSeqLengthUnknown),
                    "DeserializeUnexpectedEnd" => Ok(Error::DeserializeUnexpectedEnd),
                    "DeserializeBadVarint" => Ok(Error::DeserializeBadVarint),
                    "DeserializeBadBool" => Ok(Error::DeserializeBadBool),
                    "DeserializeBadChar" => Ok(Error::DeserializeBadChar),
                    "DeserializeBadUtf8" => Ok(Error::DeserializeBadUtf8),
                    "DeserializeBadOption" => Ok(Error::DeserializeBadOption),
                    "DeserializeBadEnum" => Ok(Error::DeserializeBadEnum),
                    "DeserializeBadEncoding" => Ok(Error::DeserializeBadEncoding),
                    "DeserializeBadCrc" => Ok(Error::DeserializeBadCrc),
                    "SerdeSerCustom" => Ok(Error::SerdeSerCustom),
                    "SerdeDeCustom" => Ok(Error::SerdeDeCustom),
                    "CollectStrError" => Ok(Error::CollectStrError),
                    _ => Err(E::unknown_variant(value, &VARIANT_NAMES)),
                }
            }
        }

        deserializer.deserialize_enum("Error", &VARIANT_NAMES, ErrorVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use core::fmt::{self, Display, Formatter};
    use serde::{Deserialize as _, Serialize as _};

    struct DisplayEnumUsingSerde(Error);

    impl Display for DisplayEnumUsingSerde {
        fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
            Error::serialize(&self.0, formatter)
        }
    }

    #[test]
    fn test_serde() {
        for i in 0.. {
            // Deserialize from integer to Error
            let de = serde::de::value::U32Deserializer::<Error>::new(i);
            let Ok(error) = Error::deserialize(de) else {
                assert_eq!(i, super::VARIANT_NAMES.len() as u32);
                break;
            };

            // Verify integer representation matches discriminant
            assert_eq!(i, error.clone() as u32);

            // Serialize from Error to integer
            let mut buf = [0u8; 1];
            crate::to_slice(&error, &mut buf).unwrap();
            assert_eq!(i, buf[0] as u32);

            // Serialize from Error to string
            let string = DisplayEnumUsingSerde(error.clone()).to_string();

            // Verify string representation matches derived Debug impl
            assert_eq!(string, format!("{error:?}"));

            // Deserialize from string to Error
            let de = serde::de::value::StrDeserializer::<Error>::new(&string);
            let error2 = Error::deserialize(de).unwrap();
            assert_eq!(error, error2);
        }
    }
}
