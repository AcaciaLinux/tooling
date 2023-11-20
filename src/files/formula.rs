//! The data structures to parse from the formula file, refer to <https://acacialinux.github.io/concept/formula> for more information
use serde::{Deserialize, Serialize};

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
    pub real_version: u32,
    pub description: String,

    pub host_dependencies: Option<Vec<String>>,
    pub target_dependencies: Option<Vec<String>>,
    pub extra_dependencies: Option<Vec<String>>,

    #[serde(default = "default_formula_package_strip")]
    pub strip: bool,

    pub arch: FormulaPackageArch,

    pub prepare: Option<String>,
    pub build: Option<String>,
    pub check: Option<String>,
    pub package: Option<String>,
}

/// The architecture parsing enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FormulaPackageArch {
    Any(String),
    Specific(Vec<String>),
}

/// Provides the default value for the `stip` field: `true`
fn default_formula_package_strip() -> bool {
    true
}
