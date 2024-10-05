use std::{
    io::{Read, Seek},
    path::PathBuf,
};

use crate::{
    error::{Error, ErrorExt},
    util::{fs::PathUtil, Packable, Unpackable},
};

use super::ObjectID;

/// A dependency needed by an object
#[derive(Debug)]
pub struct ObjectDependency {
    /// The object ID of the dependency expected
    pub oid: ObjectID,
    /// The path relative from to the depending object
    /// it expects the dependency to exist at
    pub path: PathBuf,
}

impl ObjectDependency {
    /// Infer object dependencies from a seekable stream
    /// # Arguments
    /// * `input` - The input stream to infer from
    ///
    /// This will seek `input` and leave it in a possibly random position
    pub fn infer<R: Read + Seek>(_input: &mut R) -> Result<Vec<ObjectDependency>, Error> {
        Ok(Vec::new())
    }
}

impl Packable for ObjectDependency {
    fn pack<W: std::io::prelude::Write>(&self, output: &mut W) -> Result<(), crate::error::Error> {
        let path_str = self.path.str_lossy();

        let context = || format!("Packing object dependency {} @ {}", self.oid, path_str);

        self.oid.pack(output).e_context(context)?;

        (path_str.len() as u16).pack(output).e_context(context)?;
        output.write(path_str.as_bytes()).e_context(context)?;

        Ok(())
    }
}

impl Unpackable for ObjectDependency {
    fn unpack<R: std::io::prelude::Read>(
        input: &mut R,
    ) -> Result<Option<Self>, crate::error::Error> {
        let context = || "Unpacking object dependency";

        let mut oid = [0u8; 32];
        input.read_exact(&mut oid).e_context(context)?;
        let oid = ObjectID::new(oid);

        let path_len = u16::try_unpack(input).e_context(context)?;

        let mut path = vec![0u8; path_len as usize];
        input.read_exact(&mut path).e_context(context)?;
        let path = PathBuf::from(String::from_utf8(path).e_context(context)?);

        Ok(Some(Self { oid, path }))
    }
}
