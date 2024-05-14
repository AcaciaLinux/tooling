use std::io::{Read, Seek};

use crate::{
    error::{Error, ErrorExt},
    util::{Packable, Unpackable},
};

/// The types of objects supported
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum ObjectType {
    /// Any other object
    Other = 0,
}

impl ObjectType {
    /// Infers the object type from the supplied seekable stream
    /// # Arguments
    /// * `path` - The path to the file to infer the object type of
    ///
    /// This will seek `input` and leave it in a possibly random position
    pub fn infer<R: Read + Seek>(_input: &mut R) -> Result<Self, Error> {
        Ok(Self::Other)
    }
}

impl Packable for ObjectType {
    fn pack<W: std::io::prelude::Write>(&self, output: &mut W) -> Result<(), crate::error::Error> {
        (*self as u16)
            .pack(output)
            .e_context(|| format!("Packing ObjectType {:?}", self))
    }
}

impl Unpackable for ObjectType {
    fn unpack<R: std::io::prelude::Read>(input: &mut R) -> Result<Option<Self>, crate::error::Error>
    where
        Self: Sized,
    {
        let input = u16::try_unpack(input).e_context(|| "Unpacking ObjectType")?;
        Ok(match input {
            0 => Some(Self::Other),
            _ => None,
        })
    }
}
