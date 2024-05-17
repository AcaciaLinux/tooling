use crate::{
    error::ErrorExt,
    util::{Packable, Unpackable},
};

/// The supported forms of compression applied to objects
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObjectCompression {
    /// No compression
    None = 0,
    Xz = 1,
}

impl ObjectCompression {
    pub fn from_u16(value: u16) -> Option<ObjectCompression> {
        match value {
            0 => Some(ObjectCompression::None),
            _ => None,
        }
    }
}

impl Packable for ObjectCompression {
    fn pack<W: std::io::prelude::Write>(&self, output: &mut W) -> Result<(), crate::error::Error> {
        (*self as u16)
            .pack(output)
            .e_context(|| format!("Packing {:?}", self))
    }
}

impl Unpackable for ObjectCompression {
    fn unpack<R: std::io::prelude::Read>(
        input: &mut R,
    ) -> Result<Option<Self>, crate::error::Error> {
        let input = u16::try_unpack(input).e_context(|| "Unpacking ObjectCompression")?;
        Ok(match input {
            0 => Some(Self::None),
            1 => Some(Self::Xz),
            _ => None,
        })
    }
}
