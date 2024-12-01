use crate::{
    error::{version::VersionError, Error, ErrorExt, ErrorType},
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
    /// All the dependencies of the object to aid dependency resolving
    pub dependencies: Vec<ObjectID>,
    /// The type of object contained inside
    pub ty: ObjectType,
    /// The compression applied to the inner data
    pub compression: ObjectCompression,
}

impl Object {
    /// Resolves the dependencies of this objects into objects
    /// # Arguments
    /// * `odb` - The object database to use for resolving
    /// * `recursive` - Whether to recursively resolve dependencies
    pub fn resolve_dependencies(
        &self,
        odb: &ObjectDB,
        recursive: bool,
    ) -> Result<Vec<Object>, Error> {
        let mut res = Vec::new();

        for oid in &self.dependencies {
            let object = odb
                .get_object(oid)
                .ctx(|| format!("Resolving dependency {} for {}", oid, self.oid))?;

            if recursive {
                res.append(
                    &mut object
                        .resolve_dependencies(odb, recursive)
                        .ctx(|| format!("Resolving dependency {} for {}", oid, self.oid))?,
                );
            }

            res.push(object);
        }

        Ok(res)
    }
}

impl Packable for Object {
    fn pack<W: std::io::prelude::Write>(&self, output: &mut W) -> Result<(), crate::error::Error> {
        let context = || format!("Packing object {}", self.oid.to_hex_str());

        output.write_all("AOBJ".as_bytes()).e_context(context)?;
        output.write_all(&[0]).e_context(context)?;
        self.oid.pack(output).e_context(context)?;
        self.ty.pack(output).e_context(context)?;
        self.compression.pack(output).e_context(context)?;

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
        // Read and parse the file magic ('AOBJ')
        let mut magic = [0u8; 4];
        input.read_exact(&mut magic).e_context(|| "Reading magic")?;

        if magic != *"AOBJ".as_bytes() {
            return Err(Error::new(ErrorType::Version(
                VersionError::ObjectMagicNotSupported(magic),
            )));
        }

        // Read and parse the object version
        let mut version = [0u8; 1];
        input
            .read_exact(&mut version)
            .e_context(|| "Reading version")?;

        if version[0] != 0 {
            return Err(Error::new(ErrorType::Version(
                VersionError::ObjectVersionNotSupported(version[0]),
            )));
        }

        // TODO: Think about this, this should return a None!
        let oid = ObjectID::try_unpack(input).e_context(|| "Reading object ID")?;
        let ty = ObjectType::try_unpack(input).e_context(|| "Reading object type")?;
        let compression =
            ObjectCompression::try_unpack(input).e_context(|| "Unpacking compression")?;

        let deps_count = u16::try_unpack(input).e_context(|| "Unpacking dependencies count")?;

        let mut dependencies: Vec<ObjectID> = Vec::new();

        for i in 0..deps_count {
            let dep = ObjectID::try_unpack(input).ctx(|| format!("Unpacking dependency {i}"))?;
            dependencies.push(dep);
        }

        Ok::<Option<Self>, Error>(Some(Self {
            oid,
            dependencies,
            ty,
            compression,
        }))
    }
}
