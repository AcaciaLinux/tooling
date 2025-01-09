use std::process::ExitStatus;

mod workdir;
pub use workdir::*;

use crate::error::{Error, ErrorExt, ErrorType, Throwable};

pub struct Builder {}

/// An error that originated from the `Builder` struct
#[derive(Debug)]
pub enum BuilderError {
    /// The builder tried to build a package for a non-supported architecture
    UnsupportedArch {
        arch: String,
        available_archs: Vec<String>,
    },
    /// The builder could not find a dependency for the building process
    DependencyNotFound { name: String },
    /// A subcommand failed and the builder cannot continue working
    CommandFailed { status: ExitStatus },
}

impl<T> ErrorExt<T> for Result<T, BuilderError> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::Builder(e),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for BuilderError {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::Builder(self), context)
    }
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedArch {
                arch,
                available_archs,
            } => write!(
                f,
                "Unsupported architecture '{}', available architectures: {}",
                arch,
                available_archs.join(", ")
            ),
            Self::DependencyNotFound { name } => {
                write!(f, "Dependency '{name}' not found")
            }
            Self::CommandFailed { status } => {
                write!(f, "Command failed with the following code: {}", status)
            }
        }
    }
}
