use super::CorePackage;

/// Describes a package, just the neccessary stuff
#[derive(Clone, Debug)]
pub struct PackageInfo {
    /// The name of the package
    pub name: String,
    /// The version
    pub version: String,
    /// The architecture it is built for
    pub arch: String,
}

impl CorePackage for PackageInfo {
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
