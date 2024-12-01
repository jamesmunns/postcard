use core::fmt::{self, Display};

/// Errors encountered by `postcard-dyn`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error<SerializeError> {
    Deserialize(postcard::Error),
    Serialize(SerializeError),
}

impl<SerializeError: Display> Display for Error<SerializeError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deserialize(err) => Display::fmt(err, f),
            Self::Serialize(err) => Display::fmt(err, f),
        }
    }
}

impl<SerializeError: core::error::Error> core::error::Error for Error<SerializeError> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Deserialize(err) => err.source(),
            Self::Serialize(err) => err.source(),
        }
    }
}
