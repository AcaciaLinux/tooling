use std::{collections::HashMap, path::Path};

use crate::env::EnvironmentExecutable;

use super::{Formula, FormulaPackage};

/// The type of build step at hand
#[derive(Clone, Copy)]
pub enum BuildStepType {
    Prepare,
    Build,
    Check,
    Package,
}

impl BuildStepType {
    /// Return the name of the build step in string form
    pub fn string(&self) -> &str {
        match self {
            Self::Prepare => "prepare",
            Self::Build => "build",
            Self::Check => "check",
            Self::Package => "package",
        }
    }
}

impl Formula {
    /// Returns the build step requested via `step` if available
    /// # Arguments
    /// * `step` - The step to get from the formula
    /// # Returns
    /// The command if specified or `None`
    pub fn get_build_step(&self, step: BuildStepType) -> Option<String> {
        match step {
            BuildStepType::Prepare => self.prepare.clone(),
            BuildStepType::Build => self.build.clone(),
            BuildStepType::Check => self.check.clone(),
            BuildStepType::Package => self.package.clone(),
        }
    }
}

impl FormulaPackage {
    /// Returns the build step requested via `step` if available
    /// # Arguments
    /// * `step` - The step to get from the package
    /// # Returns
    /// The command if specified or `None`
    pub fn get_build_step(&self, step: BuildStepType) -> Option<String> {
        match step {
            BuildStepType::Prepare => self.prepare.clone(),
            BuildStepType::Build => self.build.clone(),
            BuildStepType::Check => self.check.clone(),
            BuildStepType::Package => self.package.clone(),
        }
    }
}

/// A build step
pub struct BuildStep {
    /// The name of the build step
    build_step_name: String,
    /// The command to be executed
    command: String,
    /// The value to populate `PKG_NAME` with
    pkg_name: String,
    /// The value to populate `PKG_VERSION` with
    pkg_version: String,
}

impl BuildStep {
    /// Derives a build step from a formula
    /// # Arguments
    /// * `formula` - The formula to derive the build step from
    /// * `command` - The command to execute
    /// * `build_step_name` - The description and name for this build step
    pub fn new_formula(formula: &Formula, command: String, build_step_name: String) -> Self {
        Self {
            build_step_name,
            command,
            pkg_name: formula.name.clone(),
            pkg_version: formula.version.clone(),
        }
    }

    /// Creates a new build step from the provided parameters
    /// # Arguments
    /// * `command` - The command to execute
    /// * `pkg_name` - The value to populate `PKG_NAME` with
    /// * `pkg_version` - The value to populate `PKG_VERSION` with
    /// * `build_step_name` - The description and name for this build step
    pub fn new(
        command: String,
        pkg_name: String,
        pkg_version: String,
        build_step_name: String,
    ) -> Self {
        Self {
            build_step_name,
            command,
            pkg_name,
            pkg_version,
        }
    }
}

impl EnvironmentExecutable for BuildStep {
    fn get_name(&self) -> String {
        self.build_step_name.clone()
    }

    fn get_env_variables(&self) -> std::collections::HashMap<String, String> {
        let mut envs = HashMap::new();

        envs.insert("PKG_NAME", &self.pkg_name);
        envs.insert("PKG_VERSION", &self.pkg_version);

        envs.into_iter()
            .map(|k| (k.0.to_string(), k.1.to_string()))
            .collect()
    }

    fn get_command(&self) -> std::ffi::OsString {
        self.command.clone().into()
    }

    fn get_workdir(&self) -> &std::path::Path {
        Path::new("/")
    }
}
