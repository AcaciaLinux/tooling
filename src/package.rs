//! Structs and traits for managing packages
use std::{
    collections::LinkedList,
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    files::package_index::IndexPackage,
    util::fs::{Directory, SearchType},
};

mod installed;
pub use installed::*;

mod built;
pub use built::*;

mod installed_index;
pub use installed_index::*;

/// A provider for `IndexPackage`s
pub trait PackageIndexProvider<'a> {
    /// Returns the array of `IndexPackage`s the struct provides
    fn get_packages(&'a self) -> &'a [IndexPackage];
}

/// The minimal trait to be considered a package
pub trait CorePackage {
    /// Returns the `name` of the package
    fn get_name(&self) -> &str;
    /// Returns the `version` of the package
    fn get_version(&self) -> &str;
    /// Returns the `architecture` of the package
    fn get_arch(&self) -> &str;

    /// Returns the path to the package when it is installed: `<DIST_DIR>/<arch>/<name>/<version>`
    fn get_path(&self, dist_dir: &Path) -> PathBuf {
        dist_dir
            .join(self.get_arch())
            .join(self.get_name())
            .join(self.get_version())
    }

    /// Returns the full name for this package: `<arch>-<name>-<version>`
    fn get_full_name(&self) -> String {
        format!(
            "{}-{}-{}",
            self.get_arch(),
            self.get_name(),
            self.get_version()
        )
    }
}

/// A fully qualified package has a `real_version`
pub trait FQPackage: CorePackage {
    /// Returns the `real_version` of the package
    fn get_real_version(&self) -> u32;

    /// Returns the fully qualified name for this package: `<arch>-<name>-<version>-<real_version>`
    fn get_fq_name(&self) -> String {
        format!("{}-{}", self.get_full_name(), self.get_real_version())
    }
}

/// A package that is indexed and can be searched for files
pub trait IndexedPackage: CorePackage {
    /// Returns the index of the contained files starting from `<pkg_dir>/root`
    fn get_index(&self) -> &Directory;

    /// Tries to find a filesystem entry in this package
    /// # Arguments
    /// * `entry` - The entry to search for
    /// # Returns
    /// A linked list constructing the path to the found file or `None`
    fn find_fs_entry(&self, entry: &SearchType) -> Option<(LinkedList<OsString>, &Self)>
    where
        Self: Sized,
    {
        self.get_index()
            .find_entry(entry)
            .map(|entry| (entry, self))
    }
}
