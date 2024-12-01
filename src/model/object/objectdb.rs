use std::{
    fmt::Display,
    fs::File,
    io::{copy, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use log::{debug, trace};

use crate::{
    error::{Error, ErrorExt, ErrorType, Throwable},
    util::{
        fs::{self, file_create, file_open, PathUtil},
        Unpackable,
    },
    OBJECT_FILE_EXTENSION,
};

use super::{Object, ObjectCompression, ObjectID, ObjectReader, ObjectType};

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

    /// Returns the path to the internal temporary directory
    pub fn get_temp_dir(&self) -> PathBuf {
        self.get_root().join("temp")
    }

    /// Returns a path to a temporary file to use as a buffer
    pub fn get_temp_file_path(&self) -> PathBuf {
        let uuid = uuid::Uuid::new_v4();
        self.get_temp_dir().join(uuid.to_string())
    }

    /// Returns the current depth
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    /// Inserts a file and tries to infer its type and dependencies (TODO)
    ///
    /// Currently, this function does a normal [insert_file()](ObjectDB::insert_file())
    /// using the [Other](ObjectType::Other) object type and no dependencies
    /// # Arguments
    /// * `path` - The path to the file to be inserted
    /// * `compression` - The compression to use on this file
    /// * `skip_duplicate` - Whether to skip an already existing entry
    /// # Returns
    /// The inserted [Object](super::Object)
    ///
    /// This will hash the file, analyze its type and dependencies and copy it into the database
    pub fn insert_file_infer(
        &mut self,
        path: &Path,
        compression: ObjectCompression,
        skip_duplicate: bool,
    ) -> Result<Object, Error> {
        self.insert_file(
            path,
            ObjectType::Other,
            compression,
            skip_duplicate,
            Vec::new(),
        )
    }

    /// Inserts a file into the database
    /// # Arguments
    /// * `path` - The path to the file to insert
    /// * `ty` - The type of object to be inserted
    /// * `compression` - The compression to apply to the data
    /// * `skip_duplicate` - Whether to skip an already existing entry
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
        skip_duplicate: bool,
        dependencies: Vec<ObjectID>,
    ) -> Result<Object, Error> {
        let mut src_file = fs::file_open(path)?;

        let object =
            self.insert_stream(&mut src_file, ty, compression, skip_duplicate, dependencies)?;
        debug!("Inserted file {} as {}", path.str_lossy(), object.oid);

        Ok(object)
    }

    /// Insert a new object into the database by reading from a stream
    /// # Arguments
    /// * `input` - The input stream to insert
    /// * `ty` - The type of object to be inserted
    /// * `compression` - The compression to apply to the data
    /// * `skip_duplicate` - Whether to skip an already existing entry
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
        skip_duplicate: bool,
        dependencies: Vec<ObjectID>,
    ) -> Result<Object, Error> {
        input
            .seek(SeekFrom::Start(0))
            .e_context(|| "Seeking to start of stream")?;

        let oid =
            ObjectID::new_from_stream(input, &dependencies).ctx(|| "Hashing source stream")?;
        let mut db_path = self.root.join(oid.to_path(self.depth));
        db_path.set_extension(OBJECT_FILE_EXTENSION);

        if db_path.exists() {
            let mut object_file =
                fs::file_open(&db_path).e_context(|| "Opening existing object file")?;

            let object = Object::try_unpack(&mut object_file)
                .e_context(|| "Unpacking existing object file")?;

            if skip_duplicate && object.oid == oid && object.compression == compression {
                trace!("Skipping insertion of existing object {}", oid);
                return Ok(object);
            }
        }

        if let Some(p) = db_path.parent() {
            fs::create_dir_all(p)
                .e_context(|| format!("Creating parent directory {}", p.str_lossy()))?;
        }

        // Create a temporary file for inserting
        let temp_file_path = self.get_temp_file_path();

        if let Some(p) = temp_file_path.parent() {
            fs::create_dir_all(p).ctx(|| format!("Creating parent directory {}", p.str_lossy()))?;
        }

        let temp_file = fs::file_create(&temp_file_path).ctx(|| "Creating temporary file")?;

        let object = Object::create_from_stream(input, temp_file, dependencies, ty, compression)
            .ctx(|| "Inserting object")?;

        fs::rename(&temp_file_path, &db_path).ctx(|| "Moving temporary file to final place")?;

        Ok(object)
    }

    /// Tries to read an object from the database
    /// # Arguments
    /// * `oid` - The object id of the object to read
    /// # Returns
    /// `None` if the object does not exist, else an [ObjectReader](super::ObjectReader)
    pub fn try_read(&self, oid: &ObjectID) -> Result<Option<ObjectReader>, Error> {
        let mut path = self.root.join(oid.to_path(self.depth));
        path.set_extension(OBJECT_FILE_EXTENSION);

        if !path.exists() {
            return Ok(None);
        }

        let file =
            file_open(&path).e_context(|| format!("Opening object file @ {}", path.str_lossy()))?;

        let reader = ObjectReader::from_stream(file)
            .e_context(|| format!("Creating object reader for {oid}"))?;

        trace!("Retrieved object {:?}", reader.object);

        Ok(Some(reader))
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
        let mut path = self.root.join(oid.to_path(self.depth));
        path.set_extension(OBJECT_FILE_EXTENSION);

        if !path.exists() {
            return Ok(None);
        }

        let mut file =
            file_open(&path).e_context(|| format!("Opening object file @ {}", path.str_lossy()))?;

        let object = Object::unpack(&mut file).ctx(|| "Unpacking object")?;

        Ok(object)
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
}

impl Display for ObjectDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectNotFound(oid) => write!(f, "Object {oid} not found"),
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
