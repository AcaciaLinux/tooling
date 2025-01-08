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
    pub file_version: u32,

    /// The name of the formula
    pub name: String,
    /// The version of the formula
    pub version: String,
    pub description: String,

    /// The dependencies needed by the building system
    pub host_dependencies: Option<Vec<VersionString>>,
    /// The dependencies needed to run and build the target package
    pub target_dependencies: Option<Vec<VersionString>>,
    /// Additional dependencies that the system did not pick up
    pub extra_dependencies: Option<Vec<VersionString>>,

    /// A list of source needed to build the formula
    pub sources: Option<Vec<FormulaFileSource>>,

    /// The architecture the formula can be built for
    #[serde(default, deserialize_with = "deserialize_archs")]
    pub arch: Option<Vec<Architecture>>,

    /// A list of packages provided by this formula
    #[serde(default)]
    pub packages: IndexMap<String, FormulaFilePackage>,

    /// The 'prepare' build step
    pub prepare: Option<String>,
    /// The 'build' build step
    pub build: Option<String>,
    /// The 'check' build step
    pub check: Option<String>,
    /// The 'package' build step
    pub package: Option<String>,

    /// Whether or not to strip the resulting binaries
    #[serde(default = "default_formula_strip")]
    pub strip: bool,

    /// The layout of the package's output files
    #[serde(default)]
    pub layout: IndexMap<String, Vec<String>>,
}

/// A source for a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaFileSource {
    pub url: String,
    pub dest: Option<String>,

    #[serde(default = "default_formula_source_extract")]
    pub extract: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaFilePackage {
    /// The description for the package
    pub description: Option<String>,

    /// Whether or not to strip the resulting binaries
    pub strip: Option<bool>,

    /// The 'prepare' build step
    pub prepare: Option<String>,
    /// The 'build' build step
    pub build: Option<String>,
    /// The 'check' build step
    pub check: Option<String>,
    /// The 'package' build step
    pub package: Option<String>,

    /// The layout of the package's output files
    #[serde(default)]
    pub layout: IndexMap<String, Vec<String>>,
}

impl NamedPackage for FormulaFile {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl VersionedPackage for FormulaFile {
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

impl NameVersionPackage for FormulaFile {}

impl CorePackage for FormulaFile {}

/// Provides the default value for the `strip` field: `true`
fn default_formula_strip() -> bool {
    true
}

/// Provides the default value for the `extract` field: `false`
fn default_formula_source_extract() -> bool {
    false
}

impl FormulaFile {
    /// Returns the full name of the package, using the supplied architecture
    pub fn get_full_name(&self, arch: &str) -> String {
        format!("{arch}-{}-{}", self.name, self.version)
    }

    /// Returns the architectures this package can be built for
    pub fn get_architectures(&self) -> Option<Vec<Architecture>> {
        self.arch.as_ref().cloned()
    }
}

impl FormulaFileSource {
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
