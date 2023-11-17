//! The data structures to parse from the formula file, refer to <https://acacialinux.github.io/concept/formula> for more information
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// The structure that wraps the file's root contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaFile {
    /// The version of the file
    pub version: u32,
    /// There can be multiple formulae
    pub formula: HashMap<String, Formula>,
}

/// A formula
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Formula {
    pub version: String,
    pub real_version: u32,
    pub description: String,

    pub build_dependencies: Option<Vec<String>>,
    pub extra_dependencies: Option<Vec<String>>,

    pub prepare: Option<String>,
    pub build: Option<String>,
    pub check: Option<String>,
    pub package: Option<String>,

    pub sources: Option<Vec<FormulaSource>>,
    pub maintainers: Option<Vec<FormulaMaintainer>>,

    pub packages: Option<HashMap<String, FormulaPackage>>,
}

/// A source for the formula
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaSource {
    pub url: String,
    pub dest: Option<PathBuf>,
}

/// A maintainer for the formula
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaMaintainer {
    pub name: String,
    pub email: String,
}

/// A package built by the formula
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaPackage {
    pub version: Option<String>,
    pub real_version: Option<u32>,
    pub description: Option<String>,

    pub extra_dependencies: Option<Vec<String>>,

    pub prepare: Option<String>,
    pub build: Option<String>,
    pub check: Option<String>,
    pub package: Option<String>,
}

/// A populated package represents a package that inherited all inheritable
/// and non-overwritten fields from the formula
#[derive(Debug, Clone)]
pub struct PopulatedPackage {
    pub version: String,
    pub real_version: u32,
    pub description: String,

    pub extra_dependencies: Vec<String>,

    pub prepare: Option<String>,
    pub build: Option<String>,
    pub check: Option<String>,
    pub package: Option<String>,
}

impl FormulaPackage {
    /// Moves a package to a `PopulatedPackage`, inheriting all non-overwritten
    /// fields from the formula
    /// # Arguments
    /// * `formula` - The formula to inherit fields from
    pub fn populate(self, formula: &Formula) -> PopulatedPackage {
        PopulatedPackage {
            version: self.version.unwrap_or(formula.version.clone()),
            real_version: self.real_version.unwrap_or(formula.real_version),
            description: self.description.unwrap_or(formula.description.clone()),
            extra_dependencies: self
                .extra_dependencies
                .unwrap_or(formula.extra_dependencies.clone().unwrap_or(Vec::new())),
            prepare: self.prepare,
            build: self.build,
            check: self.check,
            package: self.package,
        }
    }
}
