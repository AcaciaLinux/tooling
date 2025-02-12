use std::io::Read;

use log::debug;

use crate::{
    error::{Error, ErrorType},
    model::{Object, ObjectCompression, ObjectID, ObjectReader, ObjectType, SeekRead},
};

use super::ObjectDBError;

pub mod odb_driver {
    //! Drivers for the object database
    mod odb_fs_driver;
    pub use odb_fs_driver::*;
}

/// A common trait for all object database drivers that allows layered
/// access to an object database such as over the filesystem or other sources
pub trait ODBDriver {
    /// Inserts into the underlying object database
    /// # Arguments
    /// * `object_template` - The template to create the object from
    /// * `compression` - The type of compression to use when inserting
    /// # Returns
    /// The object that was created by inserting the data
    fn insert(
        &mut self,
        object_template: ObjectTemplate,
        compression: ObjectCompression,
    ) -> Result<Object, Error>;

    /// Retrieves an object from the object database
    /// # Arguments
    /// * `oid` - The object ID of the object to retrieve
    /// # Returns
    /// The object or `None` if it is not found
    fn try_retrieve(&self, oid: &ObjectID) -> Result<Option<ObjectReader>, Error>;

    fn retrieve(&self, oid: &ObjectID) -> Result<ObjectReader, Error> {
        match self.try_retrieve(oid)? {
            None => Err(Error::new(ErrorType::ObjectDB(
                ObjectDBError::ObjectNotFound(oid.clone()),
            ))),
            Some(r) => Ok(r),
        }
    }

    /// Returns whether this driver contains the object with `oid`
    /// # Arguments
    /// * `oid` - The object id to search for
    fn exists(&self, oid: &ObjectID) -> bool;

    /// Pulls `oid` from `other`
    /// # Arguments
    /// * `other` - The object database driver to pull the data from
    /// * `oid` - The object id of the object to pull
    /// * `compression` - The compression to apply when inserting
    /// * `recursive` - Whether to operate recursively
    fn pull(
        &mut self,
        other: &dyn ODBDriver,
        oid: ObjectID,
        compression: ObjectCompression,
        recursive: bool,
    ) -> Result<(), Error> {
        let exists = self.exists(&oid);

        let object = if exists {
            debug!("[SKIP] Pulling {oid}");
            self.retrieve(&oid)?.object
        } else {
            debug!("Pulling {oid}");
            let mut object = other.retrieve(&oid)?;
            let ty = object.object.ty;
            let dependencies = object.object.dependencies.clone();

            let template = ObjectTemplate::new_prehashed(&mut object, oid, ty, dependencies);

            self.insert(template, compression)?
        };

        if recursive {
            for dependency in object.dependencies {
                self.pull(other, dependency, compression, recursive)?;
            }
        }

        Ok(())
    }
}

/// A stream that provides the data of the object to
/// the object template consumer
pub enum ObjectTemplateStream<'a> {
    /// The data is passed normally, the object id is computed
    /// by seeking the stream and hashing the data
    Normal(&'a mut dyn SeekRead),
    /// Data is already prehashed and there is no need to
    /// seek around in the stream
    Prehashed {
        /// The stream providing the data
        stream: &'a mut dyn Read,
        /// The object ID that results from hashing the stream
        oid: ObjectID,
    },
}

/// A template to create an object of by inserting it into an object database driver
pub struct ObjectTemplate<'a> {
    stream: ObjectTemplateStream<'a>,
    ty: ObjectType,
    dependencies: Vec<ObjectID>,
}

impl<'a> ObjectTemplate<'a> {
    /// Create a new object template
    /// # Arguments
    /// * `stream` - The stream to use as the object data
    /// * `ty` - The type of object at hand
    /// * `dependencies` - The dependencies of the object
    pub fn new(stream: &'a mut dyn SeekRead, ty: ObjectType, dependencies: Vec<ObjectID>) -> Self {
        Self {
            stream: ObjectTemplateStream::Normal(stream),
            ty,
            dependencies,
        }
    }

    /// Create a new object template from a prehashed stream
    /// # Arguments
    /// * `stream` - The stream to store
    /// * `oid` - The prehashed object id of the stream
    /// * `ty` - The object type at hand
    /// * `dependencies` - The dependencies of the object
    pub fn new_prehashed(
        stream: &'a mut dyn Read,
        oid: ObjectID,
        ty: ObjectType,
        dependencies: Vec<ObjectID>,
    ) -> Self {
        Self {
            stream: ObjectTemplateStream::Prehashed { stream, oid },
            ty,
            dependencies,
        }
    }

    /// Splits the template up into its stream, type and dependencies
    pub fn split_up(self) -> (ObjectTemplateStream<'a>, ObjectType, Vec<ObjectID>) {
        (self.stream, self.ty, self.dependencies)
    }
}
