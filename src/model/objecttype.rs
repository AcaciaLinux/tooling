use crate::{
    error::ErrorExt,
    util::{Packable, Unpackable},
};

/// The types of objects supported
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum ObjectType {
    /// Any other object
    Other = 0,
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
