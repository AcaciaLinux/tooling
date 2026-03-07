use std::io::Read;

use crate::{
    error::ALResult,
    odb::{ObjectCompression, ObjectID},
};

mod fs_driver;
pub use fs_driver::FsODBDriver;

pub trait ODBDriver {
    fn insert(
        &self,
        stream: &mut dyn Read,
        compression: ObjectCompression,
    ) -> ALResult<(usize, ObjectID)>;
}
