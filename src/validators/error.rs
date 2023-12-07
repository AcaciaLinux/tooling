use std::ffi::OsString;

use crate::error::{Error, ErrorExt, ErrorType, Throwable};

/// An error that occured during validation
#[derive(Debug)]
pub enum ValidationError {
    /// A file was searched but could not be found
    UnresolvedDependency { filename: OsString },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnresolvedDependency { filename } => {
                write!(f, "Unresolved dependency '{}'", filename.to_string_lossy())
            }
        }
    }
}

impl<T> ErrorExt<T> for Result<T, ValidationError> {
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(ErrorType::Validation(e), context())),
        }
    }
}

impl Throwable for ValidationError {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::Validation(self), context)
    }
}
