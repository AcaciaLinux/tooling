use super::{ALError, ALErrorExt, ALResult};

/// All the different kinds of errors that can occur in the AcaciaLinux system
#[derive(Debug)]
pub enum ALErrorKind {
    /// An io error
    IO(std::io::Error),
}

impl From<std::io::Error> for ALErrorKind {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl<T> ALErrorExt<T> for Result<T, std::io::Error> {
    fn ctx<S: ToString, F: Fn() -> S>(self, f: F) -> ALResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(ALError::new_ctx(e.into(), f)),
        }
    }
}
