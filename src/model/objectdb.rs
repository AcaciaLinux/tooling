use std::{
    io::{self, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use log::{debug, trace};

use crate::{
    error::{Error, ErrorExt},
    model::ObjectType,
    util::{
        fs::{self, PathUtil},
        hash::hash_stream,
        Packable, Unpackable,
    },
    OBJECT_FILE_EXTENSION,
};

use super::{Object, ObjectCompression, ObjectDependency, ObjectID};

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
    /// * `compression` - The compression level to use for storing the object
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

        let oid = ObjectID::from(hash_stream(&mut src_file).e_context(|| "Hashing source file")?);
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
            ObjectDependency::infer(&mut src_file).e_context(|| "Analyzing object dependencies")?;

        let ty = ObjectType::infer(&mut src_file).e_context(|| "Inferring object type")?;

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

        src_file
            .seek(SeekFrom::Start(0))
            .e_context(|| "Seeking to start of source file")?;

        io::copy(&mut src_file, &mut dst_file).e_context(|| "Copying object contents")?;

        debug!("Inserted object {}", object.oid);

        Ok(object)
    }
}
