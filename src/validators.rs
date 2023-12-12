//! Tools for validating packages and files to be able to work in the AcaciaLinux system

pub mod elf;
pub mod indexed_package;
pub mod scripts;

mod error;
use std::{collections::HashMap, path::Path, process::Command};

pub use error::*;

use crate::{
    error::Error,
    package::{CorePackage, DependencyProvider, InstalledPackageIndex, PackageInfo, PathPackage},
};

use self::indexed_package::FileValidationResult;

/// The information required for a validator to work
pub struct ValidationInput<'a> {
    /// The index of packages a validator can use for finding packages and their contents
    pub package_index: &'a InstalledPackageIndex,
    /// If the binaries should be stripped
    pub strip: bool,
}

/// The result of a validation with multiple actions and (possibly) errors
pub struct ValidationResult<T> {
    /// The actions to perform according to the validator
    pub actions: Vec<T>,
    /// The errors that occured during validation
    pub errors: Vec<Error>,
}

/// The possible actions to take according to a validator
#[derive(Clone)]
pub enum ValidatorAction {
    /// Perform an action on a `ELF` file
    ELF(elf::ELFAction),
    /// Perform an action on a `Script` file
    Script(scripts::ScriptAction),
}

impl ValidatorAction {
    /// Converts this action to a command that can be executed
    /// # Arguments
    /// * `file` - The file to execute the action on
    /// * `target_package` - The package providing the `file`
    /// * `dist_dir` - The **ABSOLUTE** path to the `dist` directory
    /// # Returns
    /// The command or an error
    pub fn to_command<T>(
        &self,
        file: &Path,
        target_package: &T,
        dist_dir: &Path,
    ) -> Result<Command, Error>
    where
        T: CorePackage + PathPackage,
    {
        match self {
            Self::ELF(a) => a.to_command(file, target_package, dist_dir),
            Self::Script(a) => a.to_command(file, target_package, dist_dir),
        }
    }
}

impl DependencyProvider for ValidatorAction {
    fn get_dependencies(&self) -> Vec<&PackageInfo> {
        match self {
            Self::ELF(e) => e.get_dependencies(),
            Self::Script(s) => s.get_dependencies(),
        }
    }
}

impl std::fmt::Display for ValidatorAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ELF(action) => action.fmt(f),
            Self::Script(action) => action.fmt(f),
        }
    }
}

/// Extracts a list of dependencies from a list of validation results
///
/// There will be no duplicates due to an internal hashmap with the full package name as the key
/// # Arguments
/// * `results` - A list of results
pub fn dependencies_from_validation_result(results: &[FileValidationResult]) -> Vec<&PackageInfo> {
    let mut res: HashMap<String, &PackageInfo> = HashMap::new();

    for result in results {
        for action in &result.actions {
            action.get_dependencies().into_iter().for_each(|d| {
                res.insert(d.get_full_name(), d);
            });
        }
    }

    res.into_iter().map(|s| s.1).collect()
}
