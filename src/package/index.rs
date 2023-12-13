//! Structs and traits for managing package indices

use super::IndexedPackage;
use crate::util::fs::SearchType;
use std::{collections::LinkedList, ffi::OsString};

mod installed;
pub use installed::*;

mod indexed;
pub use indexed::*;

/// An index of packages that can have various functions to find stuff
pub trait PackageIndex {
    /// Tries to find a filesystem entry in this package
    /// # Arguments
    /// * `entry` - The entry to search for
    /// # Returns
    /// A linked list constructing the path to the found file or `None`
    fn find_fs_entry(
        &self,
        entry: &SearchType,
    ) -> Option<(LinkedList<OsString>, &dyn IndexedPackage)>;
}

/// A collection of `PackageIndex`. This allows for layering of indices
pub struct IndexCollection<'a> {
    /// All the indices that should be searched
    pub indices: Vec<&'a dyn PackageIndex>,
}

impl<'a> PackageIndex for IndexCollection<'a> {
    fn find_fs_entry(
        &self,
        entry: &SearchType,
    ) -> Option<(LinkedList<OsString>, &dyn IndexedPackage)> {
        // Iterate over all indices to find the searched one
        for index in &self.indices {
            if let Some(res) = index.find_fs_entry(entry) {
                return Some(res);
            }
        }

        None
    }
}
