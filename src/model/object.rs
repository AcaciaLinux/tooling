use crate::{
    error::{Error, ErrorExt},
    util::{Packable, Unpackable},
};

mod objectcompression;
pub use objectcompression::*;

mod objectdb;
pub use objectdb::*;

mod objectdependency;
pub use objectdependency::*;

mod objectid;
pub use objectid::*;

mod objectreader;
pub use objectreader::*;

mod objecttype;
pub use objecttype::*;

/// A container for generic data to be handled by the AcaciaLinux system
#[derive(Debug)]
pub struct Object {
    /// The unique object ID calculated from the contents
    pub oid: ObjectID,
    /// All the dependencies of the object and where they should be placed
    pub dependencies: Vec<ObjectDependency>,
    /// The type of object contained inside
    pub ty: ObjectType,
    /// The compression applied to the inner data
    pub compression: ObjectCompression,
}

impl Packable for Object {
    fn pack<W: std::io::prelude::Write>(&self, output: &mut W) -> Result<(), crate::error::Error> {
        let context = || format!("Packing object {}", self.oid.to_hex_str());

        self.oid.pack(output).e_context(context)?;
        self.ty.pack(output).e_context(context)?;
        self.compression.pack(output).e_context(context)?;

        // GPG length
        0u16.pack(output).e_context(context)?;

        (self.dependencies.len() as u16)
            .pack(output)
            .e_context(context)?;

        for dep in &self.dependencies {
            dep.pack(output).e_context(context)?;
        }

        Ok(())
    }
}

impl Unpackable for Object {
    fn unpack<R: std::io::prelude::Read>(input: &mut R) -> Result<Option<Self>, Error> {
        let context = || "Unpacking Object";

        // TODO: Think about this, this should return a None!
        let oid = ObjectID::try_unpack(input).e_context(context)?;
        let ty = ObjectType::try_unpack(input).e_context(context)?;
        let compression = ObjectCompression::try_unpack(input).e_context(context)?;

        // Read signature
        let sig_len = u16::try_unpack(input).e_context(context)?;
        let mut buf = vec![0u8; sig_len as usize];
        input.read_exact(&mut buf).e_context(context)?;

        let deps_count = u16::try_unpack(input).e_context(context)?;

        let mut dependencies: Vec<ObjectDependency> = Vec::new();

        for _ in 0..deps_count {
            let dep = ObjectDependency::try_unpack(input).e_context(context)?;
            dependencies.push(dep);
        }

        Ok(Some(Self {
            oid,
            dependencies,
            ty,
            compression,
        }))
    }
}
