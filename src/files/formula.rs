//! The data structures to parse from the formula file, refer to <https://acacialinux.github.io/concept/formula> for more information

use serde::{Deserialize, Serialize};

use crate::{
    package::CorePackage,
    util::{parse::versionstring::VersionString, string::replace_package_variables},
    ANY_ARCH,
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
    pub pkgver: u32,
    pub description: String,

    pub host_dependencies: Option<Vec<VersionString>>,
    pub target_dependencies: Option<Vec<VersionString>>,
    pub extra_dependencies: Option<Vec<VersionString>>,

    #[serde(default = "default_formula_package_strip")]
    pub strip: bool,

    pub arch: Option<Vec<String>>,

    pub prepare: Option<String>,
    pub build: Option<String>,
    pub check: Option<String>,
    pub package: Option<String>,

    pub sources: Option<Vec<FormulaPackageSource>>,
}

/// A source for a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaPackageSource {
    pub url: String,
    pub dest: Option<String>,

    #[serde(default = "default_formula_package_source_extract")]
    pub extract: bool,
}

/// Provides the default value for the `strip` field: `true`
fn default_formula_package_strip() -> bool {
    true
}

/// Provides the default value for the `extract` field: `false`
fn default_formula_package_source_extract() -> bool {
    true
}

impl FormulaPackage {
    /// Returns the full name of the package, using the supplied architecture
    pub fn get_full_name(&self, arch: &str) -> String {
        format!("{arch}-{}-{}", self.name, self.version)
    }

    /// Returns the architectures this package can be built for
    pub fn get_architectures(&self) -> Vec<String> {
        match &self.arch {
            None => vec![ANY_ARCH.to_string()],
            Some(s) => s.clone(),
        }
    }
}

impl FormulaPackageSource {
    /// Returns the URL of the source with the variables replaced using [crate::util::string::replace_package_variables()]
    /// # Arguments
    /// * `package` - The package to pull the variables from
    pub fn get_url(&self, package: &dyn CorePackage) -> String {
        replace_package_variables(&self.url, package)
    }

    /// Returns the URL of the source with the variables replaced using [crate::util::string::replace_package_variables()]
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
