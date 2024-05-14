use std::path::PathBuf;

use crate::{
    error::ErrorExt,
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

impl Packable for ObjectDependency {
    fn pack<W: std::io::prelude::Write>(&self, output: &mut W) -> Result<(), crate::error::Error> {
        let path_str = self.path.str_lossy();

        let context = || format!("Packing object dependency {} @ {}", self.oid, path_str);

        (self.oid.len() as u32).pack(output).e_context(context)?;
        (path_str.len() as u32).pack(output).e_context(context)?;

        self.oid.pack(output).e_context(context)?;
        output.write(path_str.as_bytes()).e_context(context)?;

        Ok(())
    }
}

impl Unpackable for ObjectDependency {
    fn unpack<R: std::io::prelude::Read>(
        input: &mut R,
    ) -> Result<Option<Self>, crate::error::Error> {
        let context = || "Unpacking object dependency";

        let oid_len = u32::try_unpack(input).e_context(context)?;
        let path_len = u32::try_unpack(input).e_context(context)?;

        let mut oid = vec![0u8; oid_len as usize];
        input.read_exact(&mut oid).e_context(context)?;
        let oid = ObjectID::new(oid);

        let mut path = vec![0u8; path_len as usize];
        input.read_exact(&mut path).e_context(context)?;
        let path = PathBuf::from(String::from_utf8(path).e_context(context)?);

        Ok(Some(Self { oid, path }))
    }
}
