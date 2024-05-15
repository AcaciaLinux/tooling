use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use log::{debug, trace};

use crate::{
    error::{Error, ErrorExt},
    model::ObjectType,
    util::{
        fs::{self, file_open, PathUtil},
        hash::hash_stream,
        Packable, Unpackable,
    },
    OBJECT_FILE_EXTENSION,
};

use super::{Object, ObjectCompression, ObjectDependency, ObjectID, ObjectReader};

/// A database for storing AcaciaLinux objects
pub struct ObjectDB {
    root: PathBuf,

    depth: usize,
}

impl ObjectDB {
    /// Initializes an object database
    /// # Arguments
    /// * `root` - The directory the object database works / lives at
    /// * `depth` - The depth value for the database
    pub fn init(root: PathBuf, depth: usize) -> Result<Self, Error> {
        debug!(
            "Initializing object db @ {} (depth {})",
            root.str_lossy(),
            depth
        );

        fs::create_dir_all(&root).e_context(|| "Creating object database root")?;

        Ok(Self { root, depth })
    }

    /// Returns the root directory
    pub fn get_root(&self) -> &Path {
        &self.root
    }

    /// Returns the current depth
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    /// Inserts a file into the database
    /// # Arguments
    /// * `path` - The path to the file to insert
    /// * `compression` - The compression to apply to the data
    /// # Returns
    /// The inserted [Object](super::Object)
    ///
    /// This will hash the file, analyze its type and dependencies and copy it into the database
    pub fn insert_file(
        &mut self,
        path: &Path,
        compression: ObjectCompression,
    ) -> Result<Object, Error> {
        let mut src_file = fs::file_open(path)?;

        self.insert_stream(&mut src_file, compression)
    }

    /// Insert a new object into the database by reading from a stream
    /// # Arguments
    /// * `input` - The input stream to insert
    /// * `compression` - The compression to apply to the data
    /// # Returns
    /// The inserted [Object](super::Object)
    ///
    /// This will hash the file, analyze its type and dependencies and copy it into the database
    ///
    /// This will seek the stream and leave it at an undefined position!
    pub fn insert_stream<R: Read + Seek>(
        &mut self,
        input: &mut R,
        compression: ObjectCompression,
    ) -> Result<Object, Error> {
        input
            .seek(SeekFrom::Start(0))
            .e_context(|| "Seeking to start of stream")?;

        let oid = ObjectID::from(hash_stream(input).e_context(|| "Hashing source stream")?);
        let mut db_path = self.root.join(oid.to_path(self.depth));
        db_path.set_extension(OBJECT_FILE_EXTENSION);

        if db_path.exists() {
            trace!("Skipping insertion of existing object {}", oid);
            let mut object_file = fs::file_open(&db_path).e_context(|| "Opening object file")?;
            return Object::try_unpack(&mut object_file).e_context(|| "Unpacking object file");
        }

        if let Some(p) = db_path.parent() {
            fs::create_dir_all(p)
                .e_context(|| format!("Creating parent directory {}", p.str_lossy()))?;
        }

        let mut dst_file = fs::file_create(&db_path).e_context(|| "Creating object file")?;

        let dependencies =
            ObjectDependency::infer(input).e_context(|| "Analyzing object dependencies")?;

        let ty = ObjectType::infer(input).e_context(|| "Inferring object type")?;

        match compression {
            ObjectCompression::None => {}
        };

        let object = Object {
            oid,
            dependencies,
            ty,
            compression,
        };

        object
            .pack(&mut dst_file)
            .e_context(|| "Packing object data")?;

        input
            .seek(SeekFrom::Start(0))
            .e_context(|| "Seeking to start of source file")?;

        io::copy(input, &mut dst_file).e_context(|| "Copying object contents")?;

        debug!("Inserted object {}", object.oid);

        Ok(object)
    }

    /// Read an object from the database
    /// # Arguments
    /// * `oid` - The object id of the object to read
    /// # Returns
    /// `None` if the object does not exist, else an [ObjectReader](super::ObjectReader)
    pub fn read(&self, oid: ObjectID) -> Result<Option<ObjectReader<File>>, Error> {
        let mut path = self.root.join(oid.to_path(self.depth));
        path.set_extension(OBJECT_FILE_EXTENSION);

        if !path.exists() {
            return Ok(None);
        }

        let file =
            file_open(&path).e_context(|| format!("Opening object file @ {}", path.str_lossy()))?;

        let reader = ObjectReader::from_stream(file)
            .e_context(|| format!("Creating object reader for {oid}"))?;

        Ok(Some(reader))
    }
}
