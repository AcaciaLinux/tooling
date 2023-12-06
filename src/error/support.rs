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

impl<T> ErrorExt<T> for Result<T, toml::de::Error> {
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(ErrorType::TOML(e), context())),
        }
    }
}

impl Throwable for toml::de::Error {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::TOML(self), context)
    }
}
