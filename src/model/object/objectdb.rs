use std::{
    fmt::Display,
    fs::File,
    io::{copy, Read, Seek, SeekFrom},
    path::Path,
};

use log::{debug, trace};

use crate::{
    error::{Error, ErrorExt, ErrorType, Throwable},
    util::fs::{self, file_create, PathUtil},
};

use super::{Object, ObjectCompression, ObjectID, ObjectReader, ObjectType};

mod driver;
pub use driver::*;

/// A database for storing AcaciaLinux objects
pub struct ObjectDB {
    driver: Box<dyn ODBDriver>,
}

impl ObjectDB {
    /// Initializes an object database
    /// # Arguments
    /// * `driver` - The underlying driver for the odb to operate on top of
    pub fn init(driver: Box<dyn ODBDriver>) -> Result<Self, Error> {
        Ok(Self { driver })
    }

    /// Inserts a file and tries to infer its type and dependencies (TODO)
    ///
    /// Currently, this function does a normal [insert_file()](ObjectDB::insert_file())
    /// using the [Other](ObjectType::Other) object type and no dependencies
    /// # Arguments
    /// * `path` - The path to the file to be inserted
    /// * `compression` - The compression to use on this file
    /// # Returns
    /// The inserted [Object](super::Object)
    ///
    /// This will hash the file, analyze its type and dependencies and copy it into the database
    pub fn insert_file_infer(
        &mut self,
        path: &Path,
        compression: ObjectCompression,
    ) -> Result<Object, Error> {
        self.insert_file(path, ObjectType::Other, compression, Vec::new())
    }

    /// Inserts a file into the database
    /// # Arguments
    /// * `path` - The path to the file to insert
    /// * `ty` - The type of object to be inserted
    /// * `compression` - The compression to apply to the data
    /// * `dependencies` - The dependencies of the object to insert
    /// # Returns
    /// The inserted [Object](super::Object)
    ///
    /// This will hash the file, analyze its type and dependencies and copy it into the database
    pub fn insert_file(
        &mut self,
        path: &Path,
        ty: ObjectType,
        compression: ObjectCompression,
        dependencies: Vec<ObjectID>,
    ) -> Result<Object, Error> {
        let mut src_file = fs::file_open(path)?;

        let object = self.insert_stream(&mut src_file, ty, compression, dependencies)?;
        debug!("Inserted file {} as {}", path.str_lossy(), object.oid);

        Ok(object)
    }

    /// Insert a new object into the database by reading from a stream
    /// # Arguments
    /// * `input` - The input stream to insert
    /// * `ty` - The type of object to be inserted
    /// * `compression` - The compression to apply to the data
    /// * `dependencies` - The dependencies of the object to insert
    /// # Returns
    /// The inserted [Object](super::Object)
    ///
    /// This will hash the file, analyze its type and dependencies and copy it into the database
    ///
    /// This will seek the stream and leave it at an undefined position!
    pub fn insert_stream<R: Read + Seek>(
        &mut self,
        input: &mut R,
        ty: ObjectType,
        compression: ObjectCompression,
        dependencies: Vec<ObjectID>,
    ) -> Result<Object, Error> {
        let template = ObjectTemplate::new(input, ty, dependencies);

        self.driver.insert(template, compression)
    }

    /// Tries to read an object from the database
    /// # Arguments
    /// * `oid` - The object id of the object to read
    /// # Returns
    /// `None` if the object does not exist, else an [ObjectReader](super::ObjectReader)
    pub fn try_read(&self, oid: &ObjectID) -> Result<Option<ObjectReader>, Error> {
        self.driver.retrieve(oid)
    }

    /// Reads an object from the database
    /// # Arguments
    /// * `oid` - The object id of the object to read
    /// # Returns
    /// An [ObjectReader](super::ObjectReader) for reading object data
    pub fn read(&self, oid: &ObjectID) -> Result<ObjectReader, Error> {
        match self.try_read(oid)? {
            None => Err(Error::new(ErrorType::ObjectDB(
                ObjectDBError::ObjectNotFound(oid.clone()),
            ))),
            Some(r) => Ok(r),
        }
    }

    /// Reads an object from the database and copies it to a file
    /// # Arguments
    /// * `oid` - The object id of the object to read
    /// * `path` - The path to copy the object to
    /// # Returns
    /// A [File] seeked to the beginning to use or drop
    pub fn read_to_file(&self, oid: &ObjectID, path: &Path) -> Result<File, Error> {
        trace!("Extracting {oid} to {}", path.str_lossy());

        let mut file = file_create(path)?;
        let mut object = self.read(oid)?;

        copy(&mut object, &mut file).e_context(|| "Copying object contents")?;

        file.seek(SeekFrom::Start(0))
            .e_context(|| "Seeking to beginning of file")?;

        Ok(file)
    }

    /// Tries to get an object from the database
    /// # Arguments
    /// * `oid` - The object id of the object to read
    /// # Returns
    /// `None` if the object does not exist, else an [Object](super::Object)
    pub fn try_get_object(&self, oid: &ObjectID) -> Result<Option<Object>, Error> {
        Ok(self.driver.retrieve(oid)?.map(|o| o.object))
    }

    /// Reads an object from the database
    /// # Arguments
    /// * `oid` - The object id of the object to read
    /// # Returns
    /// An [Object](super::Object)
    pub fn get_object(&self, oid: &ObjectID) -> Result<Object, Error> {
        match self.try_get_object(oid)? {
            None => Err(Error::new(ErrorType::ObjectDB(
                ObjectDBError::ObjectNotFound(oid.clone()),
            ))),
            Some(r) => Ok(r),
        }
    }
}

/// An error that ocurred while working with the object database
#[derive(Debug)]
pub enum ObjectDBError {
    /// An object was not found in the database
    ObjectNotFound(ObjectID),
    ObjectIDMismatch {
        expected: ObjectID,
        received: ObjectID,
    },
}

impl Display for ObjectDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectNotFound(oid) => write!(f, "Object {oid} not found"),
            Self::ObjectIDMismatch { expected, received } => write!(
                f,
                "Object ID mismatch - expected {expected}, got {received}"
            ),
        }
    }
}

impl<T> ErrorExt<T> for Result<T, ObjectDBError> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::ObjectDB(e),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for ObjectDBError {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::ObjectDB(self), context)
    }
}
