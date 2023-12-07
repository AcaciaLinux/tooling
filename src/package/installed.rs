use crate::{
    error::{Error, ErrorExt},
    files::{package_index::IndexPackage, package_meta::PackageFile},
    util::{fs::Directory, parse::parse_toml},
};
use std::path::{Path, PathBuf};

use super::{BuiltPackage, CorePackage, IndexedPackage, PathPackage};

/// An installed package
#[derive(Debug)]
pub struct InstalledPackage {
    /// The name
    pub name: String,
    /// The version
    pub version: String,
    /// The architecture
    pub arch: String,
    /// The description for the package
    pub description: String,

    /// The path to where the package lives
    pub path: PathBuf,

    /// An index of all files in the package directory
    pub index: Directory,
}

impl InstalledPackage {
    /// Creates a new `InstalledPackage` from a `IndexPackage` by parsing the package metadata file and indexing the `root` directory
    /// # Arguments
    /// * `index_pkg` - The IndexPackage to use for information on where to find the package
    /// * `acacia_dir` - The path to the `/acacia` directory to search for packages
    pub fn parse_from_index(index_pkg: &IndexPackage, acacia_dir: &Path) -> Result<Self, Error> {
        let pkg_path = index_pkg.get_path(acacia_dir);

        let context = || {
            format!(
                "Parsing package {} at {}",
                index_pkg.get_full_name(),
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
            description: pkg_meta.package.description,
            path: pkg_path,
            index: dir,
        })
    }
}

impl CorePackage for InstalledPackage {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_version(&self) -> &str {
        &self.version
    }

    fn get_arch(&self) -> &str {
        &self.arch
    }
}

impl IndexedPackage for InstalledPackage {
    fn get_index(&self) -> &Directory {
        &self.index
    }
}

impl PathPackage for InstalledPackage {
    fn get_real_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl From<BuiltPackage> for InstalledPackage {
    fn from(value: BuiltPackage) -> Self {
        Self {
            name: value.name,
            version: value.version,
            arch: value.arch,
            description: value.description,
            path: value.path,
            index: value.index,
        }
    }
}