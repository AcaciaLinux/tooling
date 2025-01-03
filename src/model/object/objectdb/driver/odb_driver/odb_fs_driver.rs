use std::path::{Path, PathBuf};

use crate::{
    error::{Error, ErrorExt},
    model::{Object, ObjectCompression, ObjectID, ObjectReader},
    util::fs,
    OBJECT_FILE_EXTENSION, ODB_DEPTH,
};

use super::super::{ODBDriver, ObjectTemplate};

/// Represents an object database implemented using a filesystem tree structure
pub struct FilesystemDriver {
    root: PathBuf,
}

impl FilesystemDriver {
    /// Create a new filesystem driver that uses the filesystem
    /// to represent an object database
    /// # Arguments
    /// * `root` - The root to initialize the object database in
    pub fn new(root: PathBuf) -> Result<Self, Error> {
        fs::create_dir_all(&root).ctx(|| "Creating ODB root")?;

        Ok(Self { root })
    }

    /// Returns the root directory
    pub fn get_root(&self) -> &Path {
        &self.root
    }

    /// Returns the path to the internal temporary directory
    pub fn get_temp_dir(&self) -> PathBuf {
        self.get_root().join("temp")
    }

    /// Returns a path to a temporary file to use as a buffer
    pub fn get_temp_file_path(&self) -> PathBuf {
        let uuid = uuid::Uuid::new_v4();
        self.get_temp_dir().join(uuid.to_string())
    }

    fn get_oid_path(&self, oid: &ObjectID) -> PathBuf {
        let mut path = self.root.join(oid.to_path(ODB_DEPTH));
        path.set_extension(OBJECT_FILE_EXTENSION);

        path
    }
}

impl ODBDriver for FilesystemDriver {
    fn insert(
        &mut self,
        object_template: ObjectTemplate,
        compression: ObjectCompression,
    ) -> Result<Object, Error> {
        let temp_file_path = self.get_temp_file_path();
        fs::create_parent_dir_all(&temp_file_path)
            .ctx(|| "Creating temporary object file parent")?;

        let temp_file =
            fs::file_create(&temp_file_path).ctx(|| "Creating temporary object file")?;

        let object = Object::create_from_template(object_template, temp_file, compression)
            .ctx(|| "Creating object file")?;

        let file_path = self.get_oid_path(&object.oid);
        fs::create_parent_dir_all(&file_path).ctx(|| "Creating object parent directory")?;
        fs::copy(&temp_file_path, &file_path).ctx(|| "Copying object file to final path")?;

        Ok(object)
    }

    fn try_retrieve(&self, oid: &ObjectID) -> Result<Option<ObjectReader>, crate::error::Error> {
        let file_path = self.get_oid_path(oid);

        if !file_path.exists() {
            return Ok(None);
        }

        let file = fs::file_open(&file_path).ctx(|| "Opening object file")?;

        Ok(Some(
            ObjectReader::from_stream(file).ctx(|| "Reading object")?,
        ))
    }

    fn exists(&self, oid: &ObjectID) -> bool {
        let file_path = self.get_oid_path(oid);

        file_path.exists()
    }
}
