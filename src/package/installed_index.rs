use std::{collections::LinkedList, ffi::OsString, path::Path};

use log::debug;

use crate::{
    error::{Error, ErrorExt},
    util::fs::SearchType,
};

use super::{IndexedPackage, InstalledPackage, PackageIndexProvider};

/// A searchable index of installed packages
pub struct InstalledPackageIndex {
    /// The packages
    packages: Vec<InstalledPackage>,
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

    /// Adds a package to the index
    /// # Arguments
    /// * `package` - The package to add
    pub fn push(&mut self, package: InstalledPackage) {
        self.packages.push(package)
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
            if let Some(found_entry) = p.get_index().find_entry(entry) {
                return Some((found_entry, p));
            }
        }

        None
    }
}
