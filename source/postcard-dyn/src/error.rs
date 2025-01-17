use core::fmt::{self, Display};

/// Errors encountered by `postcard-dyn`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error<DeserializeError, SerializeError> {
    Deserialize(DeserializeError),
    Serialize(SerializeError),
}

impl<D: Display, S: Display> Display for Error<D, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deserialize(err) => Display::fmt(err, f),
            Self::Serialize(err) => Display::fmt(err, f),
        }
    }
}

impl<D: core::error::Error, S: core::error::Error> core::error::Error for Error<D, S> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Deserialize(err) => err.source(),
            Self::Serialize(err) => err.source(),
        }
    }
}
