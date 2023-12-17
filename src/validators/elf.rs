//! Validators for `ELFFile` and general ELF-related stuff
use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    assert_absolute,
    error::{Error, ErrorExt, Throwable},
    package::{CorePackage, DependencyProvider, NamedPackage, PackageInfo, PathPackage},
    util::fs::{ELFFile, ELFType, SearchType, ToPathBuf},
};

use super::{get_dest_path, ValidationError, ValidationInput, ValidationResult};

impl ELFFile {
    /// Validate an `ELFFile` (Only `ET_DYN` and `ET_EXEC`):
    /// - Make sure the `interpreter` is linkable and modify the path for the correct location
    /// - Modify the `RUNPATH`s to be able to link against shared libraries
    /// # Arguments
    /// * `input` - The `ValidationInput` to work correctly
    pub fn validate(&self, info: &ValidationInput) -> ValidationResult<ELFAction> {
        let mut actions = Vec::new();
        let mut errors = Vec::new();

        // Do only patch and strip `Dynamic` and `Executable` ELF files
        if self.ty == ELFType::Shared || self.ty == ELFType::Executable {
            // First, validate the needed so libraries
            let mut needed_runpaths: HashMap<(PathBuf, PackageInfo), ()> = HashMap::new();
            // Validate all needed shared libraries
            for so_needed in &self.shared_needed {
                if let Some(result) = info
                    .package_index
                    .find_fs_entry(&SearchType::ELF(so_needed))
                {
                    let mut path = result.0.to_path_buf();
                    path.pop();
                    needed_runpaths.insert((path, result.1.get_info()), ());
                } else {
                    errors.push(
                        ValidationError::UnresolvedDependency {
                            filename: so_needed.to_owned(),
                        }
                        .throw("Resolving needed shared object".to_string()),
                    )
                }
            }

            actions.push(ELFAction::SetRunpath {
                paths: needed_runpaths.into_keys().collect(),
            });

            // Validate the interpreter
            if let Some(interpreter) = &self.interpreter {
                if let Some(filename) = interpreter.file_name() {
                    if let Some(result) =
                        info.package_index.find_fs_entry(&SearchType::ELF(filename))
                    {
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

            // For now, if allowed by the input, always strip binaries
            if info.strip {
                actions.push(ELFAction::Strip);
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
    /// Set the RUNPATH to the supplied paths provided by the packages
    SetRunpath { paths: Vec<(PathBuf, PackageInfo)> },
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
                let dest =
                    get_dest_path(target_package, package.get_name(), dist_dir).join(interpreter);
                let mut command = Command::new("patchelf");
                command.arg("--set-interpreter");
                command.arg(dest);
                command.arg(target_package.get_real_path().join(file));
                command
            }
            Self::SetRunpath { paths } => {
                let sond: Vec<String> = paths
                    .iter()
                    .map(|p| {
                        get_dest_path(target_package, p.1.get_name(), dist_dir)
                            .join(&p.0)
                            .to_string_lossy()
                            .to_string()
                    })
                    .collect();
                let mut command = Command::new("patchelf");
                command.arg("--set-rpath");
                command.arg(sond.join(":"));
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
            Self::SetRunpath { paths } => paths.iter().map(|p| &p.1).collect(),
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
            Self::SetRunpath { paths } => {
                let paths: Vec<Cow<str>> = paths.iter().map(|p| p.0.to_string_lossy()).collect();
                write!(f, "Set ELF RUNPATH to {}", paths.join(":"))
            }
            Self::Strip => {
                write!(f, "Strip ELF file")
            }
        }
    }
}
