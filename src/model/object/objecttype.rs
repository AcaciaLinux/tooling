use std::io::{Read, Seek};

use tooling_codegen::IntoU16;

use crate::{
    error::{Error, ErrorExt},
    util::{Packable, ReprU16, Unpackable},
};

/// The types of objects supported
#[repr(u16)]
#[derive(Clone, Copy, Debug, IntoU16)]
pub enum ObjectType {
    /// Any other object
    Other = 0,

    /// An Acacia specific formula object
    AcaciaFormula = 0x0120,

    /// An Acacia specific package object
    AcaciaPackage = 0x0130,

    /// An Acacia specific index object
    AcaciaIndex = 0x0140,

    /// An Acacia specific tree object
    AcaciaTree = 0x0150,
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
        self.into_u16()
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
        Ok(Self::from_u16(input))
    }
}
