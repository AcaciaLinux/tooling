use crate::{
    error::{Error, ErrorExt},
    files::package_meta::PackageMetaFile,
    util::{fs::Directory, parse::parse_toml},
};
use std::path::{Path, PathBuf};

use super::{
    ArchitecturePackage, BuiltPackage, CorePackage, DescribedPackage, IndexedPackage,
    NameVersionPackage, NamedPackage, PackageInfo, PathPackage, VersionedPackage,
};

/// An installed package
#[derive(Debug)]
pub struct InstalledPackage {
    /// The name
    pub name: String,
    /// The version
    pub version: String,
    /// The pkgver
    pub pkgver: u32,
    /// The architecture
    pub arch: String,
    /// The description for the package
    pub description: String,

    /// The dependencies for this package
    pub dependencies: Vec<PackageInfo>,

    /// The path to where the package lives
    pub path: PathBuf,

    /// An index of all files in the package directory
    pub index: Directory,
}

impl InstalledPackage {
    /// Creates a new `InstalledPackage` from a `CorePackage` by parsing the package metadata file and indexing the `root` directory
    ///
    /// An installed package will unwind symlinks, so symlinks to ELF files get treated as ELF files to ensure
    /// discoverability by validators
    /// # Arguments
    /// * `in_pkg` - The CorePackage to use for information on where to find the package
    /// * `acacia_dir` - The path to the `/acacia` directory to search for packages
    pub fn parse_from_info(in_pkg: &dyn CorePackage, acacia_dir: &Path) -> Result<Self, Error> {
        let pkg_path = in_pkg.get_path(acacia_dir);

        let context = || {
            format!(
                "Parsing package {} at {}",
                in_pkg.get_full_name(),
                pkg_path.to_string_lossy()
            )
        };

        let pkg_meta_path = pkg_path.join("package.toml");
        let pkg_meta: PackageMetaFile = parse_toml(&pkg_meta_path).e_context(context)?;

        let mut dependencies: Vec<PackageInfo> = Vec::new();
        for (name, dep) in pkg_meta.package.dependencies {
            dependencies.push(PackageInfo {
                name,
                version: dep.req_version.version,
                pkgver: dep.req_version.pkgver,
                arch: dep.arch,
            })
        }

        let dir = Directory::index(&pkg_path.join("root"), true, true).e_context(context)?;

        Ok(Self {
            name: pkg_meta.package.name,
            version: pkg_meta.package.version,
            pkgver: pkg_meta.package.pkgver,
            arch: pkg_meta.package.arch,
            description: pkg_meta.package.description,
            dependencies,
            path: pkg_path,
            index: dir,
        })
    }
}

impl NamedPackage for InstalledPackage {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl VersionedPackage for InstalledPackage {
    fn get_version(&self) -> &str {
        &self.version
    }
    fn get_pkgver(&self) -> u32 {
        self.pkgver
    }
}

impl ArchitecturePackage for InstalledPackage {
    fn get_arch(&self) -> &str {
        &self.arch
    }
}

impl NameVersionPackage for InstalledPackage {}

impl CorePackage for InstalledPackage {}

impl IndexedPackage for InstalledPackage {
    fn get_index(&self) -> &Directory {
        &self.index
    }
}

impl DescribedPackage for InstalledPackage {
    fn get_description(&self) -> &str {
        &self.description
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
            pkgver: value.pkgver,
            arch: value.arch,
            description: value.description,
            dependencies: value.dependencies,
            path: value.path,
            index: value.index,
        }
    }
}
