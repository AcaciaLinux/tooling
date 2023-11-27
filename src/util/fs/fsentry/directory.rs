use super::{FSEntry, SearchType};
use crate::error::{Error, ErrorExt};
use std::{collections::LinkedList, ffi::OsString, path::Path};

/// Represents a directory in a filesystem index
pub struct Directory {
    /// The name of the directory
    pub name: OsString,
    /// The children this directory holds
    pub children: Vec<FSEntry>,
}

impl Directory {
    /// Creates a new directory without any children
    pub fn new(name: OsString) -> Self {
        Self {
            name,
            children: Vec::new(),
        }
    }

    /// Creates a new `Directory` and indexes the contents of `path` into it
    /// # Arguments
    /// * `path` - The path to walk
    /// * `recursive` - If this function should operate recursively
    /// # Errors
    /// Uses the std::fs::read_dir() function which will error on:
    /// - The `path` does not exist
    /// - Permission is denied
    /// - The `path` is not a directory
    pub fn index(path: &Path, recursive: bool) -> Result<Self, Error> {
        let mut index = Self {
            name: path.file_name().unwrap_or_default().to_owned(),
            children: Vec::new(),
        };

        for entry in std::fs::read_dir(path)
            .e_context(|| format!("Reading directory contents of {}", &path.to_string_lossy()))?
        {
            let entry =
                entry.e_context(|| format!("Reading entry of {}", path.to_string_lossy()))?;
            let path = entry.path();

            // Do only walk a subdirectory if it is not a symlink
            if !path.is_symlink() && path.is_dir() && recursive {
                index.children.push(FSEntry::Directory(
                    Directory::index(&path, recursive)
                        .e_context(|| format!("Indexing {}", &path.to_string_lossy()))?,
                ));
            } else {
                index.children.push(
                    FSEntry::infer(&path)
                        .e_context(|| format!("Inferring type of {}", &path.to_string_lossy()))?,
                );
            }
        }

        Ok(index)
    }

    /// Finds an entry recursively by iterating over all children, checking the name, else calling into the lower directories
    /// # Arguments
    /// * `entry` - The entry to search for
    pub fn find_entry(&self, entry: &SearchType) -> Option<LinkedList<OsString>> {
        // First, check for all children, if they contain the entry, return it immediately
        for child in &self.children {
            if entry.matches(child) {
                let mut list = LinkedList::new();
                list.push_front(child.name().to_owned());
                list.push_front(self.name.to_owned());
                return Some(list);
            }
        }

        // If no child file matched exactly, search the subdirectories
        for child in &self.children {
            if let FSEntry::Directory(child) = child {
                if let Some(mut p) = child.find_entry(entry) {
                    if !self.name.is_empty() {
                        p.push_front(self.name.to_owned());
                    }
                    return Some(p);
                }
            }
        }

        None
    }
}
