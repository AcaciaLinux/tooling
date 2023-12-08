use super::{Error, ErrorExt, ErrorType, Throwable};

impl<T> ErrorExt<T> for Result<T, std::io::Error> {
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(ErrorType::IO(e), context())),
        }
    }
}

impl Throwable for std::io::Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::IO(self), context)
    }
}

impl<T> ErrorExt<T> for Result<T, elf::ParseError> {
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(ErrorType::ELFParse(e), context())),
        }
    }
}

impl Throwable for elf::ParseError {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::ELFParse(self), context)
    }
}

/// A TOML error
#[derive(Debug)]
pub enum TOMLError {
    /// Serialization errors
    Serialize(toml::ser::Error),
    /// Deserialization errors
    Deserialize(toml::de::Error),
}

impl std::fmt::Display for TOMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(e) => write!(f, "Serialization error: {e}"),
            Self::Deserialize(e) => write!(f, "Deserialization error: {e}"),
        }
    }
}

impl<T> ErrorExt<T> for Result<T, toml::de::Error> {
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::TOML(TOMLError::Deserialize(e)),
                context(),
            )),
        }
    }
}

impl Throwable for toml::de::Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::TOML(TOMLError::Deserialize(self)), context)
    }
}

impl<T> ErrorExt<T> for Result<T, toml::ser::Error> {
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::TOML(TOMLError::Serialize(e)),
                context(),
            )),
        }
    }
}

impl Throwable for toml::ser::Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::TOML(TOMLError::Serialize(self)), context)
    }
}
