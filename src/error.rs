//! Common error structure used all over the tooling

use std::{collections::LinkedList, string::FromUtf8Error};

use architecture::ArchitectureError;

use crate::model::ObjectDBError;
#[cfg(feature = "builder")]
use crate::tools::builder::BuilderError;

use self::{
    assert::AssertionError,
    dependency::DependencyError,
    support::{CURLError, TOMLError},
};

pub mod support;

pub mod architecture;
pub mod assert;
pub mod dependency;

/// The type of error at hand
#[derive(Debug)]
pub enum ErrorType {
    Assert(AssertionError),
    IO(std::io::Error),
    ELFParse(elf::ParseError),
    TOML(TOMLError),
    #[cfg(feature = "builder")]
    Builder(BuilderError),
    CURL(CURLError),
    Dependency(DependencyError),
    Architecture(ArchitectureError),
    FromUTF8(FromUtf8Error),
    XzStream(xz::stream::Error),
    ObjectDB(ObjectDBError),
    Other(String),
}

/// The error struct, containing the error and a context
#[derive(Debug)]
pub struct Error {
    /// A stack of contexes the error occured in
    pub context: LinkedList<String>,
    /// The error itself
    pub error: ErrorType,
}

/// Traits for handling error contexts
pub trait ErrorExt<T> {
    /// Adds context to an error. This function takes a trait, so strings do only get constructed when needed
    /// # Arguments
    /// * `context` - A closure that returns the context message
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error>;
}

/// A trait for types that can be populated to an `Error`
pub trait Throwable {
    /// Converts `self` to an `Error` with the supplied context
    fn throw(self, context: String) -> Error;
}

impl Error {
    /// Creates a new `Error`
    /// # Arguments
    /// * `error` - The error to use as a basis for the message
    pub fn new(error: ErrorType) -> Self {
        Self {
            context: LinkedList::new(),
            error,
        }
    }

    /// Creates a new `Error` with context
    /// # Arguments
    /// * `error` - The error to use as a basis for the message
    /// * `context` - The initial context message
    pub fn new_context(error: ErrorType, context: String) -> Self {
        Self {
            context: {
                let mut l = LinkedList::new();
                l.push_back(context);
                l
            },
            error,
        }
    }

    /// Converts this error to the `Err` variant of a `Result`, this will **always** return `Err`
    pub fn throw(error: ErrorType, context: String) -> Result<(), Error> {
        Err(Self::new_context(error, context))
    }

    /// Provides a oneline error message
    pub fn oneline(&self) -> String {
        self.error.to_string()
    }
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assert(e) => e.fmt(f),
            Self::IO(e) => e.fmt(f),
            Self::ELFParse(e) => e.fmt(f),
            Self::TOML(e) => e.fmt(f),
            #[cfg(feature = "builder")]
            Self::Builder(e) => e.fmt(f),
            Self::CURL(e) => e.fmt(f),
            Self::Dependency(e) => e.fmt(f),
            Self::Architecture(e) => e.fmt(f),
            Self::FromUTF8(e) => e.fmt(f),
            Self::XzStream(e) => e.fmt(f),
            Self::ObjectDB(e) => e.fmt(f),
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} while", self.error)?;
        for (i, context) in self.context.iter().enumerate() {
            write!(f, "\n{}-- {}:", "  ".repeat(i), context)?
        }
        write!(f, "\n{}-- {}", "  ".repeat(self.context.len()), self.error)
    }
}

impl<T> ErrorExt<T> for Result<T, Error> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(mut e) => {
                e.context.push_front(context().to_string());
                Err(e)
            }
        }
    }
}
