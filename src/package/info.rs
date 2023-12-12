use super::{ArchitecturePackage, CorePackage, NameVersionPackage, NamedPackage, VersionedPackage};

/// Describes a package, just the neccessary stuff
#[derive(Clone, Debug)]
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
