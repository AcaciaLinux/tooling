use std::{collections::LinkedList, ffi::OsString, path::Path};

use crate::{
    error::{dependency::DependencyError, Error, Throwable},
    util::{fs::SearchType, parse::versionstring::VersionString},
    ANY_ARCH,
};

use super::{
    super::{CorePackage, IndexedPackage, InstalledPackage, PackageInfo},
    PackageIndex,
};

/// A searchable index of installed packages
#[derive(Default)]
pub struct InstalledPackageIndex {
    /// The packages
    packages: Vec<InstalledPackage>,
}

impl InstalledPackageIndex {
    /// Creates an installed package index from a list of packages to use and a search directory
    ///
    /// TODO: Rudimentary dependency resolving, should probably be replaced with a real one...
    /// # Arguments
    /// * `list` - The list of dependencies to search for
    /// * `arch` - The preferred architecture
    /// * `search_dir` - The directory to search for (dest_dir)
    pub fn from_package_list(
        list: &[VersionString],
        arch: String,
        search_dir: &Path,
    ) -> Result<Self, Error> {
        let mut res = Self {
            packages: Vec::new(),
        };

        let any_arch = ANY_ARCH.to_owned();
        let mut additional_deps: Vec<PackageInfo> = Vec::new();

        for version_string in list {
            // First, try an architecture-specific package
            let spec_info = PackageInfo::from_version_string(version_string.clone(), arch.clone());
            let info = if spec_info.get_path(search_dir).exists() {
                spec_info
            } else {
                PackageInfo::from_version_string(version_string.clone(), any_arch.clone())
            };

            if !info.get_path(search_dir).exists() {
                return Err(DependencyError::Unresolved {
                    arch,
                    name: info.name,
                    version: info.version,
                    pkgver: info.pkgver,
                }
                .throw("Finding installed packages".to_owned()));
            }

            let package = InstalledPackage::parse_from_info(&info, search_dir)?;
            additional_deps.append(&mut package.dependencies.clone());

            res.packages.push(package);
        }

        while let Some(dep) = additional_deps.pop() {
            if !dep.get_path(search_dir).exists() {
                return Err(DependencyError::Unresolved {
                    arch,
                    name: dep.name,
                    version: dep.version,
                    pkgver: dep.pkgver,
                }
                .throw("Finding dependencies of installed packages".to_owned()));
            }

            let package = InstalledPackage::parse_from_info(&dep, search_dir)?;
            for subdependency in &package.dependencies {
                if !additional_deps.contains(subdependency) {
                    additional_deps.push(subdependency.clone());
                }
            }

            res.packages.push(package);
        }

        Ok(res)
    }

    /// Adds a package to the index
    /// # Arguments
    /// * `package` - The package to add
    pub fn push(&mut self, package: InstalledPackage) {
        self.packages.push(package)
    }

    /// Returns a reference to the inner vector if installed packages
    pub fn inner(&self) -> &Vec<InstalledPackage> {
        &self.packages
    }
}

impl PackageIndex for InstalledPackageIndex {
    fn find_fs_entry(
        &self,
        entry: &SearchType,
    ) -> Option<(LinkedList<OsString>, &dyn IndexedPackage)> {
        for p in &self.packages {
            if let Some(found_entry) = p.get_index().find_entry(entry) {
                return Some((found_entry, p));
            }
        }

        None
    }
}
