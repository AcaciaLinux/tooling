//! The data structures to parse from the formula file, refer to <https://acacialinux.github.io/concept/formula> for more information

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    package::{CorePackage, NameVersionPackage, NamedPackage, VersionedPackage},
    util::{
        architecture::{deserialize_archs, Architecture},
        parse::versionstring::VersionString,
        string::replace_package_variables,
    },
};

/// The contents of a formula file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaFile {
    /// The version of the file
    pub version: u32,
    /// There can be multiple formulae
    pub package: FormulaPackage,
}

/// A package built by the formula
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaPackage {
    pub name: String,
    pub version: String,
    pub description: String,

    pub host_dependencies: Option<Vec<VersionString>>,
    pub target_dependencies: Option<Vec<VersionString>>,
    pub extra_dependencies: Option<Vec<VersionString>>,

    #[serde(default = "default_formula_package_strip")]
    pub strip: bool,

    #[serde(default, deserialize_with = "deserialize_archs")]
    pub arch: Option<Vec<Architecture>>,

    pub prepare: Option<String>,
    pub build: Option<String>,
    pub check: Option<String>,
    pub package: Option<String>,

    pub sources: Option<Vec<FormulaPackageSource>>,

    #[serde(default)]
    pub layout: IndexMap<String, Vec<String>>,
}

/// A source for a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaPackageSource {
    pub url: String,
    pub dest: Option<String>,

    #[serde(default = "default_formula_package_source_extract")]
    pub extract: bool,
}

impl NamedPackage for FormulaPackage {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl VersionedPackage for FormulaPackage {
    fn get_version(&self) -> &str {
        &self.version
    }

    fn get_pkgver(&self) -> u32 {
        0
    }

    fn get_id(&self) -> &str {
        todo!()
    }
}

impl NameVersionPackage for FormulaPackage {}

impl CorePackage for FormulaPackage {}

/// Provides the default value for the `strip` field: `true`
fn default_formula_package_strip() -> bool {
    true
}

/// Provides the default value for the `extract` field: `false`
fn default_formula_package_source_extract() -> bool {
    false
}

impl FormulaPackage {
    /// Returns the full name of the package, using the supplied architecture
    pub fn get_full_name(&self, arch: &str) -> String {
        format!("{arch}-{}-{}", self.name, self.version)
    }

    /// Returns the architectures this package can be built for
    pub fn get_architectures(&self) -> Option<Vec<Architecture>> {
        self.arch.as_ref().cloned()
    }
}

impl FormulaPackageSource {
    /// Returns the URL of the source with the variables replaced using [crate::util::string::replace_package_variables()]
    /// # Arguments
    /// * `package` - The package to pull the variables from
    pub fn get_url(&self, package: &dyn CorePackage) -> String {
        replace_package_variables(&self.url, package)
    }

    /// Returns the destination of the source with the variables replaced using [crate::util::string::replace_package_variables()]
    /// # Arguments
    /// * `package` - The package to pull the variables from
    pub fn get_dest(&self, package: &dyn CorePackage) -> String {
        let dest = match &self.dest {
            Some(d) => d.to_owned(),
            None => self
                .get_url(package)
                .split('/')
                .last()
                .unwrap_or("download")
                .to_owned(),
        };

        replace_package_variables(&dest, package)
    }
}
