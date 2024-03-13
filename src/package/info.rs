use crate::util::parse::versionstring::VersionString;

use super::{CorePackage, NameVersionPackage, NamedPackage, VersionedPackage};

/// Describes a package, just the neccessary stuff
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PackageInfo {
    /// The name of the package
    pub name: String,
    /// The version
    pub version: String,
    /// The package version
    pub pkgver: u32,
}

impl PackageInfo {
    /// Creates a `PackageInfo` struct from a `VersionString`
    /// # Arguments
    /// * `version_string` - The source struct
    pub fn from_version_string(version_string: VersionString) -> Self {
        Self {
            name: version_string.name,
            version: version_string.version,
            pkgver: version_string.pkgver,
        }
    }

    /// Create a `PackageInfo` from a `NameVersionPackage`
    pub fn from_package(package: &dyn NameVersionPackage) -> Self {
        Self {
            name: package.get_name().to_owned(),
            version: package.get_version().to_owned(),
            pkgver: package.get_pkgver(),
        }
    }
}

impl From<&dyn CorePackage> for PackageInfo {
    fn from(value: &dyn CorePackage) -> Self {
        Self {
            name: value.get_name().to_owned(),
            version: value.get_version().to_owned(),
            pkgver: value.get_pkgver(),
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

impl NameVersionPackage for PackageInfo {}

impl CorePackage for PackageInfo {}
