use std::{
    io::{Read, Write},
    os::unix::fs::MetadataExt,
};

use crate::{
    error::{Error, ErrorExt},
    util::{Packable, Unpackable},
};

/// A structure to wrap UNIX file attributes
#[derive(Debug)]
pub struct UNIXInfo {
    /// The UNIX user id for the entry
    pub uid: u32,
    /// The UNIX group id for the entry
    pub gid: u32,
    /// The UNIX mode for the entry
    pub mode: u32,
}

impl UNIXInfo {
    /// Creates a new instance
    /// # Arguments
    /// * `uid` - The user id
    /// * `gid` - The group id
    /// * `mode` - The entry mode
    pub fn new(uid: u32, gid: u32, mode: u32) -> Self {
        Self { uid, gid, mode }
    }

    /// Creates a new instance by getting information from `entry`
    /// # Arguments
    /// * `entry` - The entry to use for getting information
    ///
    /// Uses [std::fs::DirEntry::metadata()]
    pub fn from_entry(entry: &std::fs::DirEntry) -> Result<Self, std::io::Error> {
        let metadata = entry.metadata()?;

        let uid = metadata.uid();
        let gid = metadata.gid();
        let mode = metadata.mode();

        Ok(Self { uid, gid, mode })
    }
}

impl Packable for UNIXInfo {
    fn pack<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        let context = || format!("Packing UNIX info {:?}", self);

        output.write(&self.uid.to_le_bytes()).e_context(context)?;
        output.write(&self.gid.to_le_bytes()).e_context(context)?;
        output.write(&self.mode.to_le_bytes()).e_context(context)?;

        Ok(())
    }
}

impl Unpackable for UNIXInfo {
    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self>, Error> {
        let context = || "Unpacking UNIX info";

        let mut buf = [0u8; 3 * 4];
        input.read_exact(&mut buf).e_context(context)?;

        Ok(Some(Self {
            uid: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
            gid: u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
            mode: u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
        }))
    }
}
