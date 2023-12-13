use crate::util::parse::versionstring::VersionString;

use super::{ArchitecturePackage, CorePackage, NameVersionPackage, NamedPackage, VersionedPackage};

/// Describes a package, just the neccessary stuff
#[derive(Clone, Debug, PartialEq)]
pub struct PackageInfo {
    /// The name of the package
    pub name: String,
    /// The version
    pub version: String,
    /// The package version
    pub pkgver: u32,
    /// The architecture it is built for
    pub arch: String,
}

impl PackageInfo {
    /// Creates a `PackageInfo` struct from a `VersionString`
    /// # Arguments
    /// * `version_string` - The source struct
    /// * `arch` - The archtecture to use
    pub fn from_version_string(version_string: VersionString, arch: String) -> Self {
        Self {
            name: version_string.name,
            version: version_string.version,
            pkgver: version_string.pkgver,
            arch,
        }
    }

    /// Create a `PackageInfo` from a `NameVersionPackage` and an architecture
    pub fn from_package_arch(package: &dyn NameVersionPackage, arch: String) -> Self {
        Self {
            name: package.get_name().to_owned(),
            version: package.get_version().to_owned(),
            pkgver: package.get_pkgver(),
            arch,
        }
    }
}

impl From<&dyn CorePackage> for PackageInfo {
    fn from(value: &dyn CorePackage) -> Self {
        Self {
            name: value.get_name().to_owned(),
            version: value.get_version().to_owned(),
            pkgver: value.get_pkgver(),
            arch: value.get_arch().to_owned(),
        }
    }
}

impl NamedPackage for PackageInfo {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl VersionedPackage for PackageInfo {
    fn get_version(&self) -> &str {
        &self.version
    }
    fn get_pkgver(&self) -> u32 {
        self.pkgver
    }
}

impl ArchitecturePackage for PackageInfo {
    fn get_arch(&self) -> &str {
        &self.arch
    }
}

impl NameVersionPackage for PackageInfo {}

impl CorePackage for PackageInfo {}
