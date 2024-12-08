use std::io::{Read, Seek, SeekFrom, Write};

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

    /// Creates an object from an object template
    /// # Arguments
    /// * `input` - The stream to read the object data from
    /// * `output` - The stream to write the (compressed) object data to
    /// * `compression` - The compression to apply when inserting the stream
    pub fn create_from_template<W: Write + Seek>(
        input: ObjectTemplate,
        output: W,
        compression: ObjectCompression,
    ) -> Result<Self, Error> {
        let (stream, ty, dependencies) = input.split_up();

        match stream {
            ObjectTemplateStream::Normal(stream) => {
                Self::create_from_stream(stream, output, dependencies, ty, compression)
            }
            ObjectTemplateStream::Prehashed { stream, oid } => {
                Self::create_from_prehashed(stream, oid, output, dependencies, ty, compression)
            }
        }
    }

    /// Creates an object by reading in prehashed data
    /// # Arguments
    /// * `input` - The input stream to read from
    /// * `oid` - The prehashed object id to expect from `input` (gets checked)
    /// * `output` - The output stream to write to
    /// * `dependencies` - The dependencies of the new object
    /// * `ty` - The type of object at hand
    /// * `compression` - The compression to apply when savin to `output`
    pub fn create_from_prehashed<W: Write>(
        input: &mut dyn Read,
        oid: ObjectID,
        mut output: W,
        dependencies: Vec<ObjectID>,
        ty: ObjectType,
        compression: ObjectCompression,
    ) -> Result<Self, Error> {
        let object = Self {
            oid: oid.clone(),
            dependencies,
            ty,
            compression,
        };

        object.pack_header(&mut output)?;

        let output: Box<dyn Write> = match compression {
            ObjectCompression::None => Box::new(output),
            ObjectCompression::Xz => {
                let stream = xz::stream::Stream::new_easy_encoder(6, xz::stream::Check::None)
                    .ctx(|| "Creating xz stream")?;

                Box::new(xz::write::XzEncoder::new_stream(output, stream))
            }
        };

        let mut output = ObjectIDHasher::new(output, &object.dependencies);

        std::io::copy(input, &mut output).ctx(|| "Copying object contents")?;

        let (_, hashed_oid) = output.finalize();

        // Check that the calculated OID matches the prehashed one
        if hashed_oid != oid {
            return Err(Error::new(ErrorType::ObjectDB(
                ObjectDBError::ObjectIDMismatch {
                    expected: oid,
                    received: hashed_oid,
                },
            )));
        }

        Ok(object)
    }

    /// Creates a new object from a stream and creates an object file
    /// # Arguments
    /// * `input` - The input stream to use as the object data
    /// * `output` - The output stream to write the object file's contents to
    /// * `ty` - The type of object at hand
    /// * `compression` - The type of compression to use when inserting the data
    pub fn create_from_stream<W: Write + Seek>(
        input: &mut dyn SeekRead,
        mut output: W,
        dependencies: Vec<ObjectID>,
        ty: ObjectType,
        compression: ObjectCompression,
    ) -> Result<Self, Error> {
        input
            .seek(SeekFrom::Start(0))
            .ctx(|| "Seeking to start of input stream")?;

        // First, hash the stream
        let oid =
            ObjectID::new_from_stream(input, &dependencies).ctx(|| "Calculating object id")?;

        let object = Self {
            oid,
            dependencies,
            ty,
            compression,
        };

        object.pack_header(&mut output)?;

        let mut output: Box<dyn Write> = match compression {
            ObjectCompression::None => Box::new(output),
            ObjectCompression::Xz => {
                let stream = xz::stream::Stream::new_easy_encoder(6, xz::stream::Check::None)
                    .ctx(|| "Creating xz stream")?;

                Box::new(xz::write::XzEncoder::new_stream(output, stream))
            }
        };

        input
            .seek(SeekFrom::Start(0))
            .ctx(|| "Seeking back to input start")?;

        std::io::copy(input, &mut output).e_context(|| "Copying object contents")?;

        Ok(object)
    }

    /// Packs the object header contents to `output`
    /// # Arguments
    /// * `output` - The stream to write to
    fn pack_header<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        output
            .write_all("AOBJ".as_bytes())
            .ctx(|| "Writing object magic")?;
        output.write_all(&[0]).ctx(|| "Writing object version")?;
        self.oid.pack(output).ctx(|| "Writing object ID")?;
        self.ty.pack(output).ctx(|| "Writing object type")?;
        self.compression
            .pack(output)
            .ctx(|| "Writing object compression")?;

        (self.dependencies.len() as u16)
            .pack(output)
            .ctx(|| "Packing dependencies count")?;

        for dep in &self.dependencies {
            dep.pack(output).ctx(|| "Writing dependency")?;
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
