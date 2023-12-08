//! Validators for `ELFFile` and general ELF-related stuff
use std::path::PathBuf;

use crate::{
    error::Throwable,
    package::{CorePackage, PackageInfo},
    util::fs::{ELFFile, SearchType, ToPathBuf},
};

use super::{ValidationError, ValidationInput, ValidationResult};

impl ELFFile {
    /// Validate an `ELFFile`:
    /// - Make sure the `interpreter` is linkable and modify the path for the correct location
    /// - Modify the `RUNPATH`s to be able to link against shared libraries
    /// # Arguments
    /// * `input` - The `ValidationInput` to work correctly
    pub fn validate(&self, info: &ValidationInput) -> ValidationResult<ELFAction> {
        let mut actions = Vec::new();
        let mut errors = Vec::new();

        // Validate the interpreter
        if let Some(interpreter) = &self.interpreter {
            if let Some(filename) = interpreter.file_name() {
                if let Some(result) = info.package_index.find_fs_entry(&SearchType::ELF(filename)) {
                    actions.push(ELFAction::SetInterpreter {
                        interpreter: result.0.to_path_buf(),
                        package: result.1.get_info(),
                    })
                } else {
                    errors.push(
                        ValidationError::UnresolvedDependency {
                            filename: interpreter.as_os_str().to_owned(),
                        }
                        .throw("Resolving ELF interpreter".to_string()),
                    )
                }
            }
        }

        // Validate all needed shared libraries
        for so_needed in &self.shared_needed {
            if let Some(result) = info
                .package_index
                .find_fs_entry(&SearchType::ELF(so_needed))
            {
                let mut path = result.0.to_path_buf();
                path.pop();
                actions.push(ELFAction::AddRunPath {
                    runpath: path,
                    package: result.1.get_info(),
                })
            } else {
                errors.push(
                    ValidationError::UnresolvedDependency {
                        filename: so_needed.to_owned(),
                    }
                    .throw("Resolving needed shared object".to_string()),
                )
            }
        }

        ValidationResult { actions, errors }
    }
}

/// An action to perform on an ELF file
#[derive(Clone)]
pub enum ELFAction {
    /// Set the `interpreter` available in `package`
    SetInterpreter {
        /// The interpreter to set
        interpreter: PathBuf,
        /// The package holding the interpreter (the dependency)
        package: PackageInfo,
    },
    /// Add a `runpath` available in `package`
    AddRunPath {
        /// The RUNPATH to add
        runpath: PathBuf,
        /// The package holding the RUNPATH (the dependency)
        package: PackageInfo,
    },
}

impl std::fmt::Display for ELFAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetInterpreter {
                interpreter,
                package,
            } => {
                write!(
                    f,
                    "Set ELF interpreter to '{}' of '{}'",
                    interpreter.to_string_lossy(),
                    package.get_full_name()
                )
            }
            Self::AddRunPath { runpath, package } => {
                write!(
                    f,
                    "Add to ELF RUNPATH: '{}' of '{}'",
                    runpath.to_string_lossy(),
                    package.get_full_name()
                )
            }
        }
    }
}
