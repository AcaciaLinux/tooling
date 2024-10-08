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
    /// Packs `self` into a binary stream
    /// # Arguments
    /// * `output` - The stream to write to
    fn pack<W: Write>(&self, output: &mut W) -> Result<(), Error>;
}

pub trait Unpackable {
    /// Unpacks `Self` from a binary stream
    /// # Arguments
    /// * `input` - The stream to read from
    /// # Returns
    /// `None` if the file stream ended before everything has been parsed
    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self>, Error>
    where
        Self: Sized;

    /// Tries to unpack `Self` from a binary stream, throwing an error on EOF
    /// # Arguments
    /// * `input` - The stream to read from
    fn try_unpack<R: Read>(input: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
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

/// Represents a enum as a [u16].
///
/// Commonly used for the [`IntoU16`](tooling_codegen::IntoU16) macro
pub trait ReprU16 {
    /// Returns the `u16` representation of the enum variant at hand
    fn into_u16(&self) -> u16;

    /// Returs the matching enum variant that is meant for `num`
    /// # Arguments
    /// * `num` - The number to derive the enum variant from
    /// # Returns
    /// The enum variant or `None` if there is no matching enum variant
    fn from_u16(num: u16) -> Option<Self>
    where
        Self: Sized;
}
