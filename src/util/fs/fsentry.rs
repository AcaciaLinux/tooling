use super::unwind_symlinks;
use crate::error::{Error, ErrorExt};
use log::trace;
use std::{
    collections::LinkedList,
    ffi::{OsStr, OsString},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

mod directory;
pub use directory::*;

mod elf;
pub use elf::*;

/// A filesystem entry
pub enum FSEntry {
    /// An ELF file
    ELF(ELFFile),
    /// A symlink
    Symlink(OsString),
    /// Some other file
    OtherFile(OsString),
    /// A directory
    Directory(Directory),
}

/// The type of filesystem entry to search for
pub enum SearchType<'a> {
    /// Search for an ELF file
    ELF(&'a OsStr),
    /// Search for any file
    Any(&'a OsStr),
}

impl FSEntry {
    /// Tries to infer the type of filesystem entry by using the `infer` crate
    /// # Arguments
    /// * `path` - The path to infer
    pub fn infer(path: &Path) -> Result<Self, Error> {
        let path = unwind_symlinks(path);
        let name = path.file_name().expect("Filename").to_owned();

        if path.is_symlink() {
            trace!("[infer] SLNK: {}", path.to_string_lossy());
            Ok(Self::Symlink(name))
        } else if path.is_dir() {
            trace!("[infer] DIR : {}", path.to_string_lossy());
            Ok(Self::Directory(Directory::new(name)))
        } else {
            if let Ok(mut file) = File::open(&path) {
                let mut buf = vec![0; 53];
                if file.read(&mut buf).is_ok() && infer::app::is_elf(&buf) {
                    trace!("[infer] ELF : {}", &path.to_string_lossy());

                    let f = ELFFile::parse(&path)
                        .e_context(|| format!("Parsing ELF file {}", path.to_string_lossy()))?;

                    return Ok(Self::ELF(f));
                }
            }

            trace!("[infer] FILE: {}", path.to_string_lossy());
            Ok(Self::OtherFile(
                path.file_name().expect("Filename").to_owned(),
            ))
        }
    }

    /// Returns the name of the FSEntry
    pub fn name(&self) -> &OsStr {
        match self {
            Self::ELF(n) => &n.name,
            Self::Symlink(n) => n,
            Self::OtherFile(n) => n,
            Self::Directory(d) => &d.name,
        }
    }
}

impl<'a> SearchType<'a> {
    /// Returns the name of the SearchType
    pub fn name(&self) -> &OsStr {
        match self {
            Self::ELF(n) => n,
            Self::Any(n) => n,
        }
    }

    /// Returns if this `SearchType` matches on the supplied `FSEntry`, including checking the name
    /// # Arguments
    /// * `fsentry` - The entry to check for a match
    pub fn matches(&self, fsentry: &FSEntry) -> bool {
        // Immediately sort out all entries whose name does not match
        if fsentry.name() != self.name() {
            return false;
        }

        // Now match over the FSEntry type
        match self {
            SearchType::ELF(_) => matches!(fsentry, FSEntry::ELF(_)),
            SearchType::Any(_) => true,
        }
    }
}

/// A trait to convert some other struct to a PathBuf
pub trait ToPathBuf {
    /// Converts to a `PathBuf`
    fn to_path_buf(&self) -> PathBuf;
}

impl ToPathBuf for LinkedList<OsString> {
    fn to_path_buf(&self) -> PathBuf {
        let mut path = PathBuf::new();
        for e in self {
            path.push(e);
        }
        path
    }
}
