use std::io::Read;

use crate::{
    error::ALResult,
    odb::{ObjectCompression, ObjectID, driver::ODBDriver},
};

pub struct ObjectDatabase {
    driver: Box<dyn ODBDriver>,
}

impl ObjectDatabase {
    pub fn new(driver: Box<dyn ODBDriver>) -> Self {
        Self { driver }
    }

    pub fn insert<R: Read>(&self, stream: &mut R) -> ALResult<(usize, ObjectID)> {
        self.driver.insert(stream, ObjectCompression::LZMA)
    }
}
