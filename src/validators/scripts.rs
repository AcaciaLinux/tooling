//! Validators for `Script`s
use std::path::PathBuf;

use crate::{
    error::{Error, Throwable},
    package::{CorePackage, DependencyProvider, PackageInfo},
    util::fs::{ScriptFile, SearchType, ToPathBuf},
};

use super::{ValidationError, ValidationInput, ValidationResult};

impl ScriptFile {
    /// Validate an `Script`:
    /// - Make sure the `interpreter` is linkable and modify the path for the correct location
    /// # Arguments
    /// * `input` - The `ValidationInput` to work correctly
    pub fn validate(&self, input: &ValidationInput) -> ValidationResult<ScriptAction> {
        let mut actions: Vec<ScriptAction> = Vec::new();
        let mut errors: Vec<Error> = Vec::new();

        // Validate the interpreter
        if let Some(old_interpreter) = &self.interpreter {
            if let Some(filename) = old_interpreter.0.file_name() {
                if let Some(result) = input
                    .package_index
                    .find_fs_entry(&SearchType::ELF(filename))
                {
                    actions.push(ScriptAction::ReplaceInterpreter {
                        interpreter: (old_interpreter.0.clone(), result.0.to_path_buf()),
                        package: result.1.get_info(),
                    })
                } else {
                    errors.push(
                        ValidationError::UnresolvedDependency {
                            filename: old_interpreter.0.as_os_str().to_owned(),
                        }
                        .throw("Resolving script interpreter".to_string()),
                    )
                }
            }
        }

        ValidationResult { actions, errors }
    }
}

/// An action to perform on a Script file
#[derive(Clone)]
pub enum ScriptAction {
    /// Set the `interpreter` available in `package`
    ReplaceInterpreter {
        /// The old and the new interpreter
        interpreter: (PathBuf, PathBuf),
        /// The package holding the interpreter (the dependency)
        package: PackageInfo,
    },
}

impl DependencyProvider for ScriptAction {
    fn get_dependencies(&self) -> Vec<&PackageInfo> {
        match self {
            Self::ReplaceInterpreter {
                interpreter: _,
                package,
            } => vec![package],
        }
    }
}

impl std::fmt::Display for ScriptAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReplaceInterpreter {
                interpreter,
                package,
            } => {
                write!(
                    f,
                    "Replace SCRIPT interpreter '{}' with '{}' of package '{}' (args: '{}')",
                    interpreter.0.to_string_lossy(),
                    interpreter.1.to_string_lossy(),
                    package.get_full_name(),
                    interpreter
                        .1
                        .iter()
                        .map(|s| s.to_string_lossy().to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                )
            }
        }
    }
}
