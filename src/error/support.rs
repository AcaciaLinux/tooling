//! Support code to wrap other errors in `tooling::Error` structs
use std::string::FromUtf8Error;

use http::StatusCode;

use super::{dependency::DependencyError, AssertionError, Error, ErrorExt, ErrorType, Throwable};

impl<T> ErrorExt<T> for Result<T, AssertionError> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::Assert(e),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for AssertionError {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::Assert(self), context)
    }
}

impl<T> ErrorExt<T> for Result<T, std::io::Error> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(ErrorType::IO(e), context().to_string())),
        }
    }
}

impl Throwable for std::io::Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::IO(self), context)
    }
}

impl<T> ErrorExt<T> for Result<T, elf::ParseError> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::ELFParse(e),
                context().to_string(),
            )),
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
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::TOML(TOMLError::Deserialize(e)),
                context().to_string(),
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
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::TOML(TOMLError::Serialize(e)),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for toml::ser::Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::TOML(TOMLError::Serialize(self)), context)
    }
}

impl<T> ErrorExt<T> for Result<T, serde_json::Error> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::JSON(e),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for serde_json::Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::JSON(self), context)
    }
}

/// A CURL error
#[derive(Debug)]
pub enum CURLError {
    /// An error happened in the CURL library
    CURL(curl::Error),
    /// An unknown status code has been responded
    InvalidStatus(u32),
    /// Failed request
    ErrorStatus(StatusCode),
}

impl std::fmt::Display for CURLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CURL(e) => e.fmt(f),
            Self::InvalidStatus(status) => write!(f, "Unknown HTTP response status '{}'", status),
            Self::ErrorStatus(code) => write!(f, "Request failed: {}", code),
        }
    }
}

impl<T> ErrorExt<T> for Result<T, curl::Error> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::CURL(CURLError::CURL(e)),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for curl::Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::CURL(CURLError::CURL(self)), context)
    }
}

impl<T> ErrorExt<T> for Result<T, DependencyError> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::Dependency(e),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for DependencyError {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::Dependency(self), context)
    }
}

impl<T> ErrorExt<T> for Result<T, FromUtf8Error> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::FromUTF8(e),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for FromUtf8Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::FromUTF8(self), context)
    }
}

impl<T> ErrorExt<T> for Result<T, xz::stream::Error> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::XzStream(e),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for xz::stream::Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::XzStream(self), context)
    }
}
