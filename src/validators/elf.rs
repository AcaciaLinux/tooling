//! Validators for `ELFFile` and general ELF-related stuff
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    assert_absolute,
    error::{Error, ErrorExt, Throwable},
    package::{CorePackage, DependencyProvider, NamedPackage, PackageInfo, PathPackage},
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

        // For now, if allowed by the input, always strip binaries
        if info.strip {
            actions.push(ELFAction::Strip);
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
    /// Strip the binary
    Strip,
}

impl ELFAction {
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
        let dist_dir = assert_absolute!(dist_dir)
            .e_context(|| format!("Converting action \"{}\" to executable command", self))?;

        Ok(match self {
            Self::SetInterpreter {
                interpreter,
                package,
            } => {
                let dest = target_package
                    .get_path(dist_dir)
                    .join("link")
                    .join(package.get_name())
                    .join(interpreter);
                let mut command = Command::new("patchelf");
                command.arg("--set-interpreter");
                command.arg(dest);
                command.arg(target_package.get_real_path().join(file));
                command
            }
            Self::AddRunPath { runpath, package } => {
                let dest = target_package
                    .get_path(dist_dir)
                    .join("link")
                    .join(package.get_name())
                    .join(runpath);
                let mut command = Command::new("patchelf");
                command.arg("--add-rpath");
                command.arg(dest);
                command.arg(target_package.get_real_path().join(file));
                command
            }
            Self::Strip => {
                let mut command = Command::new("strip");
                command.arg(target_package.get_real_path().join(file));
                command
            }
        })
    }
}

impl DependencyProvider for ELFAction {
    fn get_dependencies(&self) -> Vec<&PackageInfo> {
        match self {
            Self::SetInterpreter {
                interpreter: _,
                package,
            } => vec![package],
            Self::AddRunPath {
                runpath: _,
                package,
            } => vec![package],
            Self::Strip => Vec::new(),
        }
    }
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
            Self::Strip => {
                write!(f, "Strip ELF file")
            }
        }
    }
}
