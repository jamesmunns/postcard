#[cfg(feature = "alloc")]
extern crate alloc;

use core::fmt::{Display, Formatter};

/// This is the error type used by Postcard
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// This is a feature that PostCard will never implement
    WontImplement,
    /// This is a feature that Postcard intends to support, but does not yet
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
    /// Found an enum discriminant that was > u32::max_value()
    DeserializeBadEnum,
    /// The original data was not well encoded
    DeserializeBadEncoding,
    /// Serde Serialization Error
    SerdeSerCustom(Message),
    /// Serde Deserialization Error
    SerdeDeCustom(Message),
    /// Error while processing `collect_str` during serialization
    CollectStrError,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
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
                SerdeSerCustom(message) =>
                    return write!(f, "Serde Serialization Error: {}", message),
                SerdeDeCustom(message) =>
                    return write!(f, "Serde Deserialization Error: {}", message),
                CollectStrError => "Error while processing `collect_str` during serialization",
            }
        )
    }
}

/// A custom error message
///
/// Only contains an actual message when the `alloc` feature is enabled, otherwise it is empty.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Message {
    #[cfg(feature = "alloc")]
    message: alloc::string::String,
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        #[cfg(feature = "alloc")]
        let result = f.write_str(&self.message);

        #[cfg(not(feature = "alloc"))]
        let result =
            f.write_str("[Message not stored unless postcard is built with the `alloc` feature]");

        result
    }
}

/// This is the Result type used by Postcard.
pub type Result<T> = ::core::result::Result<T, Error>;

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        #[cfg(feature = "alloc")]
        {
            use core::fmt::Write;
            let mut message = alloc::string::String::new();
            write!(message, "{}", msg).unwrap();
            Error::SerdeSerCustom(Message { message })
        }

        #[cfg(not(feature = "alloc"))]
        {
            let _ = msg;
            Error::SerdeSerCustom(Message {})
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        #[cfg(feature = "alloc")]
        {
            use core::fmt::Write;
            let mut message = alloc::string::String::new();
            write!(message, "{}", msg).unwrap();
            Error::SerdeSerCustom(Message { message })
        }

        #[cfg(not(feature = "alloc"))]
        {
            let _ = msg;
            Error::SerdeSerCustom(Message {})
        }
    }
}

impl serde::ser::StdError for Error {}
