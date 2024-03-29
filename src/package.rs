//! Structs and traits for managing packages
use std::{
    collections::LinkedList,
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::util::fs::{Directory, SearchType};

use self::info::PackageInfo;

pub mod info;

/// A package that has a name
pub trait NamedPackage {
    /// Returns the `name` of the package
    fn get_name(&self) -> &str;
}

/// A package that has a version
pub trait VersionedPackage {
    /// Returns the `version` of the package
    fn get_version(&self) -> &str;
    /// Returns the `pkgver` of the package
    fn get_pkgver(&self) -> u32;
    /// Returns the unique package id
    fn get_id(&self) -> &str;
}

/// A package that provides only a name and a version
pub trait NameVersionPackage: NamedPackage + VersionedPackage {
    /// Returns the name and version for this package: `<name>-<version>-<pgkver>`
    fn get_name_version(&self) -> String {
        format!(
            "{}-{}-{}",
            self.get_name(),
            self.get_version(),
            self.get_pkgver()
        )
    }
}

/// The minimal trait to be considered a package
pub trait CorePackage: NamedPackage + VersionedPackage + NameVersionPackage {
    /// Returns the path to the package when it is installed: `<DIST_DIR>/<arch>/<name>/<version>/<pkgver>`
    fn get_path(&self, dist_dir: &Path) -> PathBuf {
        dist_dir
            .join(self.get_name())
            .join(self.get_version())
            .join(self.get_pkgver().to_string())
    }

    /// Generates a `PackageInfo` from this package to provide a portable description
    fn get_info(&self) -> PackageInfo {
        PackageInfo {
            name: self.get_name().to_owned(),
            version: self.get_version().to_owned(),
            pkgver: self.get_pkgver(),
            id: self.get_id().to_owned(),
        }
    }

    /// Returns the directory this package lives at relative to `dist_dir`
    /// # Arguments
    /// * `dist_dir` - The DIST directory
    fn get_install_dir(&self, dist_dir: &Path) -> PathBuf {
        dist_dir
            .join("pkg")
            .join(format!("{}_{}", self.get_name_version(), self.get_id()))
    }

    /// Returns the directory that contains the package files relative to `dist_dir`
    /// # Arguments
    /// * `dist_dir` - The DIST directory
    fn get_root_dir(&self, dist_dir: &Path) -> PathBuf {
        self.get_install_dir(dist_dir).join("root")
    }

    /// Returns the directory that contains the directory for linking
    /// to other objects relative to `dist_dir`
    /// # Arguments
    /// * `dist_dir` - The DIST directory
    fn get_link_dir(&self, dist_dir: &Path) -> PathBuf {
        self.get_install_dir(dist_dir).join("link")
    }
}

/// A package that has a description
pub trait DescribedPackage {
    /// Get the description for the package
    fn get_description(&self) -> &str;
}

/// A package that has a path to where it lives
pub trait PathPackage {
    /// Returns the **real** path to the package without constructing it from a DIST directory
    fn get_real_path(&self) -> PathBuf;
}

/// Something that can provide a list of dependencies
pub trait DependencyProvider {
    /// Returns all the needed dependencies
    fn get_dependencies(&self) -> Vec<&PackageInfo>;
}

/// A package that is indexed and can be searched for files
pub trait IndexedPackage: CorePackage + PathPackage {
    /// Returns the index of the contained files starting from `<pkg_dir>/root`
    fn get_index(&self) -> &Directory;

    /// Returns the directores containing executable files
    fn get_executable_dirs(&self) -> &[PathBuf];

    /// Returns the directoreis containing library files
    fn get_library_dirs(&self) -> &[PathBuf];

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
