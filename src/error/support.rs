use super::{Error, ErrorExt, ErrorType};

impl<T> ErrorExt<T> for Result<T, std::io::Error> {
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(ErrorType::IO(e), context())),
        }
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
