//! Various utility functions, structs and traits

use crate::error::{Error, ErrorType};
use std::io::{self, Read, Write};

pub mod architecture;
pub mod archive;
pub mod download;
pub mod elf;
pub mod fs;
pub mod hash;
pub mod parse;
pub mod serde;
pub mod signal;
pub mod string;

#[cfg(feature = "mount")]
pub mod mount;

/// A trait for binary packable structures
pub trait Packable {
    /// Should always be set to `Self`
    type Output;

    /// Packs `self` into a binary stream
    /// # Arguments
    /// * `output` - The stream to write to
    fn pack<W: Write>(&self, output: &mut W) -> Result<(), Error>;

    /// Unpacks `Self` from a binary stream
    /// # Arguments
    /// * `input` - The stream to read from
    /// # Returns
    /// `None` if the file stream ended before everything has been parsed
    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self::Output>, Error>;

    /// Tries to unpack `Self` from a binary stream, throwing an error on EOF
    /// # Arguments
    /// * `input` - The stream to read from
    fn try_unpack<R: Read>(input: &mut R) -> Result<Self::Output, Error> {
        let x = Self::unpack(input)?;
        match x {
            Some(x) => Ok(x),
            None => Err(Error::new(ErrorType::IO(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Unexpected EOF (end of file) while unpacking struct from binary stream",
            )))),
        }
    }
}
