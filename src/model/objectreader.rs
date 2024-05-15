use std::io::{Read, Seek};

use log::debug;

use crate::{
    error::{Error, ErrorExt},
    util::Unpackable,
};

use super::{Object, ObjectCompression};

/// A wrapper for reading (possibly) compressed object data from an object
pub struct ObjectReader {
    /// The object wrapped by this reader
    pub object: Object,
    /// The read stream
    read: Box<dyn Read>,
}

pub trait SeekRead: Seek + Read {}
impl<T: Seek + Read> SeekRead for T {}

impl ObjectReader {
    /// Parses object data from a stream and constructs a reader
    /// # Arguments
    /// * `read` - The input stream to read from
    pub fn from_stream<R: SeekRead + 'static>(mut read: R) -> Result<Self, Error> {
        let object = Object::try_unpack(&mut read).e_context(|| "Unpacking object")?;

        let read: Box<dyn Read> = match object.compression {
            ObjectCompression::None => Box::new(read),
            ObjectCompression::Xz => {
                debug!("Using XZ decompression");
                Box::new(xz::read::XzDecoder::new(read))
            }
        };

        Ok(Self { object, read })
    }
}

impl Read for ObjectReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read.read(buf)
    }
}
