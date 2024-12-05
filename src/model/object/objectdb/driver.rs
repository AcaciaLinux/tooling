use crate::{
    error::Error,
    model::{Object, ObjectCompression, ObjectID, ObjectReader, ObjectType, SeekRead},
};

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
    fn retrieve(&self, oid: &ObjectID) -> Result<Option<ObjectReader>, Error>;
}

/// A template to create an object of by inserting it into an object database driver
pub struct ObjectTemplate<'a> {
    stream: &'a mut dyn SeekRead,
    ty: ObjectType,
    dependencies: Vec<ObjectID>,
}

impl<'a> ObjectTemplate<'a> {
    /// Create a new object database driver
    /// * `stream` - The stream to use as the object data
    /// * `ty` - The type of object at hand
    /// * `dependencies` - The dependencies of the object
    pub fn new(stream: &'a mut dyn SeekRead, ty: ObjectType, dependencies: Vec<ObjectID>) -> Self {
        Self {
            stream,
            ty,
            dependencies,
        }
    }
}
