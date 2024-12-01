use std::{
    fs::{File, Permissions},
    io::{self, Read, Write},
    os::{
        fd::AsRawFd,
        unix::{
            self,
            fs::{MetadataExt, PermissionsExt},
        },
    },
    path::Path,
};

use nix::sys::stat::{FchmodatFlags, Mode};

use crate::{
    error::{Error, ErrorExt},
    util::{Packable, Unpackable},
};

/// A structure to wrap UNIX file attributes
#[derive(Debug, PartialEq, Eq)]
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

    /// Creates a new instance by getting information about `path`
    /// # Arguments
    /// * `path` - The path to analyze
    ///
    /// Uses [std::fs::metadata()]
    pub fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        let metadata = std::fs::metadata(path)?;

        let uid = metadata.uid();
        let gid = metadata.gid();
        let mode = metadata.mode();

        Ok(Self { uid, gid, mode })
    }

    /// Applies this unix information to a file path
    /// # Arguments
    /// * `path` - The path to apply the information to
    pub fn apply_path(&self, path: &Path) -> Result<(), Error> {
        unix::fs::lchown(path, Some(self.uid), Some(self.gid))
            .e_context(|| format!("Changing ownership to {}:{}", self.uid, self.gid))?;

        match nix::sys::stat::fchmodat(
            None,
            path,
            Mode::from_bits_retain(self.mode),
            FchmodatFlags::NoFollowSymlink,
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(io::Error::from(e)),
        }
        .e_context(|| format!("Changing ownership to {}:{}", self.uid, self.gid))?;

        Ok(())
    }

    /// Applies this unix information to an open file
    /// # Arguments
    /// * `file` - The file to apply to
    pub fn apply_file(&self, file: &mut File) -> Result<(), Error> {
        file.set_permissions(Permissions::from_mode(self.mode))
            .e_context(|| format!("Setting mode to {:o}", self.mode))?;

        match nix::unistd::fchown(
            file.as_raw_fd(),
            Some(self.uid.into()),
            Some(self.gid.into()),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(io::Error::from(e)),
        }
        .e_context(|| format!("Changing ownership to {}:{}", self.uid, self.gid))?;

        Ok(())
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
