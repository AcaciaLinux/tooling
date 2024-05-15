use std::io::{Read, Seek, SeekFrom};

use crate::{
    error::{Error, ErrorExt},
    util::Unpackable,
};

use super::Object;

/// A wrapper for reading (possibly) compressed object data from an object
pub struct ObjectReader<R: Read> {
    /// The object wrapped by this reader
    pub object: Object,
    /// The read stream
    read: R,
    /// The offset at which the object data starts
    data_start: u64,
}

impl<R: Read + Sized + Seek> ObjectReader<R> {
    /// Parses object data from a stream and constructs a reader
    /// # Arguments
    /// * `read` - The input stream to read from
    pub fn from_stream(mut read: R) -> Result<Self, Error> {
        let object = Object::try_unpack(&mut read).e_context(|| "Unpacking object")?;

        let data_start = read
            .stream_position()
            .e_context(|| "Getting stream position")?;

        Ok(Self {
            object,
            read,
            data_start,
        })
    }
}

impl<R: Read + Sized + Seek> Seek for ObjectReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => self.read.seek(SeekFrom::Start(self.data_start + offset)),
            SeekFrom::Current(offset) => self.read.seek(SeekFrom::Current(offset)),
            SeekFrom::End(offset) => self.read.seek(SeekFrom::End(offset)),
        }
    }
}

impl<R: Read + Sized + Seek> Read for ObjectReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read.read(buf)
    }
}
