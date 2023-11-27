use crate::{
    error::{Error, ErrorExt},
    files::{package_index::IndexPackage, package_meta::PackageFile},
    util::{
        fs::{Directory, SearchType},
        parse::parse_toml,
    },
};
use log::debug;
use std::{
    collections::LinkedList,
    ffi::OsString,
    path::{Path, PathBuf},
};

use super::PackageIndexProvider;

/// An installed package
#[derive(Debug)]
pub struct InstalledPackage {
    /// The name
    pub name: String,
    /// The version
    pub version: String,
    /// The architecture
    pub arch: String,
    /// The real version of the installed package
    pub real_version: u32,
    /// The description for the package
    pub description: String,

    /// An index of all files in the package directory
    pub directory: Option<Directory>,
}

/// A searchable index of installed packages
pub struct InstalledPackageIndex {
    /// The packages
    packages: Vec<InstalledPackage>,
}

impl InstalledPackage {
    /// Creates a new `InstalledPackage` from a `IndexPackage` by parsing the package metadata file and indexing the `root` directory
    /// # Arguments
    /// * `index_pkg` - The IndexPackage to use for information on where to find the package
    /// * `acacia_dir` - The path to the `/acacia` directory to search for packages
    pub fn parse_from_index(index_pkg: &IndexPackage, acacia_dir: &Path) -> Result<Self, Error> {
        let pkg_path = index_pkg.path(acacia_dir);

        let context = || {
            format!(
                "Parsing package {} at {}",
                index_pkg.full_name(),
                pkg_path.to_string_lossy()
            )
        };

        let pkg_meta_path = pkg_path.join("package.toml");
        let pkg_meta: PackageFile = parse_toml(&pkg_meta_path).e_context(context)?;

        let dir = Directory::index(&pkg_path.join("root"), true).e_context(context)?;

        Ok(Self {
            name: pkg_meta.package.name,
            version: pkg_meta.package.version,
            arch: pkg_meta.package.arch,
            real_version: pkg_meta.package.real_version,
            description: pkg_meta.package.description,
            directory: Some(dir),
        })
    }

    /// Returns the full name of the package in the following format: `<arch>/<name>-<version>-<real_version>`
    pub fn full_name(&self) -> String {
        format!(
            "{}/{}-{}-{}",
            self.arch, self.name, self.version, self.real_version
        )
    }

    /// Returns the path to the package directory based on the supplied `acacia_dir`
    /// # Arguments
    /// * `acacia_dir` - The acacia directory to base the package path on
    pub fn path(&self, acacia_dir: &Path) -> PathBuf {
        acacia_dir
            .join(&self.arch)
            .join(&self.name)
            .join(&self.version)
    }
}

impl InstalledPackageIndex {
    /// Creates an index of installed packages from a index provider and an acacia directory to search for packages
    /// # Arguments
    /// * `index` - The `PackageIndexProvider` to parse from
    /// * `acacia_dir` - The directory to search for packages
    pub fn from_index<'a>(
        index: &'a dyn PackageIndexProvider<'a>,
        acacia_dir: &Path,
    ) -> Result<Self, Error> {
        let context = || {
            format!(
                "Creating installed package index of {}",
                acacia_dir.to_string_lossy()
            )
        };

        let mut self_ = Self {
            packages: Vec::new(),
        };

        for package in index.get_packages() {
            let path = package.path(acacia_dir);

            if !path.exists() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Not Found: Package '{}' @ {}",
                        package.full_name(),
                        path.to_string_lossy()
                    ),
                ))
                .e_context(context)?;
            } else {
                let pkg =
                    InstalledPackage::parse_from_index(package, acacia_dir).e_context(context)?;
                debug!("Found package '{:?}' at {}", pkg, path.to_string_lossy(),);

                self_.packages.push(pkg);
            }
        }

        Ok(self_)
    }

    /// Tries to find a filesystem entry in this package
    /// # Arguments
    /// * `entry` - The entry to search for
    /// # Returns
    /// A linked list constructing the path to the found file or `None`
    pub fn find_fs_entry(
        &self,
        entry: &SearchType,
    ) -> Option<(LinkedList<OsString>, &InstalledPackage)> {
        for p in &self.packages {
            if let Some(dir) = &p.directory {
                if let Some(found_entry) = dir.find_entry(entry) {
                    return Some((found_entry, p));
                }
            }
        }

        None
    }
}
