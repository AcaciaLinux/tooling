//! Structs and traits for managing packages
use std::{
    collections::LinkedList,
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    util::fs::{Directory, SearchType},
    validators::{
        indexed_package::{validate_indexed_package, FileValidationResult},
        ValidationInput,
    },
    PACKAGE_ARCHIVE_FILE_SUFFIX,
};

mod installed;
pub use installed::*;

mod built;
pub use built::*;

pub mod index;

mod installable;
pub use installable::*;

mod info;
pub use info::*;

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
}

/// A package that has an architecture
pub trait ArchitecturePackage {
    /// Returns the `architecture` of the package
    fn get_arch(&self) -> &str;
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
pub trait CorePackage:
    NamedPackage + VersionedPackage + ArchitecturePackage + NameVersionPackage
{
    /// Returns the path to the package when it is installed: `<DIST_DIR>/<arch>/<name>/<version>/<pkgver>`
    fn get_path(&self, dist_dir: &Path) -> PathBuf {
        dist_dir
            .join(self.get_arch())
            .join(self.get_name())
            .join(self.get_version())
            .join(self.get_pkgver().to_string())
    }

    /// Returns the full name for this package: `<arch>-<name>-<version>-<pkgver>`
    fn get_full_name(&self) -> String {
        format!("{}-{}", self.get_arch(), self.get_name_version())
    }

    /// Generates a `PackageInfo` from this package to provide a portable description
    fn get_info(&self) -> PackageInfo {
        PackageInfo {
            name: self.get_name().to_owned(),
            version: self.get_version().to_string(),
            pkgver: self.get_pkgver(),
            arch: self.get_arch().to_string(),
        }
    }

    /// Returns the name for the archive file for this package
    fn get_archive_name(&self) -> String {
        format!("{}{}", self.get_full_name(), PACKAGE_ARCHIVE_FILE_SUFFIX)
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

/// Something that can provide a build id
pub trait BuildIDProvider {
    /// Returns the build id for this object
    fn get_build_id(&self) -> &str;
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

    /// Validates this package by iterating over its index and validating everything
    /// # Arguments
    /// * `input` - The validation input
    /// # Returns
    /// A vector of file results. If a file has no actions and no errors, it will not be returned
    fn validate(&self, input: &ValidationInput) -> Vec<FileValidationResult>
    where
        Self: Sized,
    {
        validate_indexed_package(self, input)
    }
}
