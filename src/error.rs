//! Common error structure used all over the tooling

use std::collections::LinkedList;

mod support;

/// The type of error at hand
#[derive(Debug)]
pub enum ErrorType {
    IO(std::io::Error),
    ELFParse(elf::ParseError),
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
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error>;
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
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::IO(e) => e.fmt(f),
            ErrorType::ELFParse(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed while")?;
        for (i, context) in self.context.iter().enumerate() {
            write!(f, "\n{}-- {}:", "  ".repeat(i), context)?
        }
        write!(f, "\n{}-- {}", "  ".repeat(self.context.len()), self.error)
    }
}

impl<T> ErrorExt<T> for Result<T, Error> {
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(mut e) => {
                e.context.push_front(context());
                Err(e)
            }
        }
    }
}