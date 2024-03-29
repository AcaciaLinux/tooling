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
    /// The unique package id
    pub id: String,
}

impl PackageInfo {
    /// Create a `PackageInfo` from a `NameVersionPackage`
    pub fn from_package(package: &dyn NameVersionPackage) -> Self {
        Self {
            name: package.get_name().to_owned(),
            version: package.get_version().to_owned(),
            pkgver: package.get_pkgver(),
            id: package.get_id().to_owned(),
        }
    }
}

impl From<&dyn CorePackage> for PackageInfo {
    fn from(value: &dyn CorePackage) -> Self {
        Self {
            name: value.get_name().to_owned(),
            version: value.get_version().to_owned(),
            pkgver: value.get_pkgver(),
            id: value.get_id().to_owned(),
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
    fn get_id(&self) -> &str {
        &self.id
    }
}

impl NameVersionPackage for PackageInfo {}

impl CorePackage for PackageInfo {}
