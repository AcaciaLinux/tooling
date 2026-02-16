use super::{ALErrorContext, ALErrorKind};

/// A Error in the AcaciaLinux tooling
#[derive(Debug)]
pub struct ALError {
    /// The kind of error at hand
    pub kind: ALErrorKind,
    /// The context the error occurred in
    pub context: ALErrorContext,
}

/// A shorthand for [Result<T, ALError>]
pub type ALResult<T> = Result<T, ALError>;

/// Trait for extended functionality that can be added to foreign types via this trait
pub trait ALErrorExt<T> {
    /// Enrich the error with some context in case of the [Err](Result::Err) variant.
    fn ctx<S: ToString, F: Fn() -> S>(self, f: F) -> ALResult<T>;
}

impl ALError {
    /// Create a new error with no context
    /// # Arguments
    /// * `kind` - The kind of error at hand
    pub fn new(kind: ALErrorKind) -> Self {
        Self {
            kind,
            context: Default::default(),
        }
    }

    /// Create a new error with some context attached to it
    /// # Arguments
    /// * `kind` - The kind of error at hand
    /// * `f` - The function to call to get the context string
    pub fn new_ctx<S: ToString, F: Fn() -> S>(kind: ALErrorKind, f: F) -> Self {
        let mut context = ALErrorContext::default();
        context.add(f);
        Self { kind, context }
    }
}

impl<T> ALErrorExt<T> for ALResult<T> {
    fn ctx<S: ToString, F: Fn() -> S>(self, f: F) -> ALResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(mut e) => {
                e.context.add(f);
                Err(e)
            }
        }
    }
}
