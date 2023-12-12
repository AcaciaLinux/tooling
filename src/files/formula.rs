//! The data structures to parse from the formula file, refer to <https://acacialinux.github.io/concept/formula> for more information
use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    env::EnvironmentExecutable,
    package::{CorePackage, NameVersionPackage, NamedPackage, VersionedPackage},
    util::string::replace_package_variables,
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
    pub description: String,

    pub host_dependencies: Option<Vec<String>>,
    pub target_dependencies: Option<Vec<String>>,
    pub extra_dependencies: Option<Vec<String>>,

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

    /// Returns the build steps for building this package
    /// # Arguments
    /// * `formula_mountpoint` - The path where the formula's parent directory is mounted **inside** the chroot
    /// * `architecture` - The architecture to build the package for (this is checked and returns an error if the architecture is not supported)
    /// * `install_dir` - The directory to install the package into **inside** the chroot (`<package_name>/data`)
    /// # Returns
    /// A vector of `PackageBuildStep`s or `None` if the architecture is not supported
    pub fn get_buildsteps(
        &self,
        formula_mountpoint: &Path,
        architecture: &str,
        install_dir: &Path,
    ) -> Option<Vec<PackageBuildStep>> {
        // Check if the architecture is supported
        if let Some(archs) = &self.arch {
            if !archs.contains(&architecture.to_owned()) {
                return None;
            }
        }

        let mut res: Vec<PackageBuildStep> = Vec::new();

        // Create the environment variables
        let env_vars = PackageEnvVars {
            env_pkg_name: self.name.clone(),
            env_pkg_version: self.version.clone(),
            env_pkg_arch: architecture.to_owned(),
            env_pkg_install_dir: install_dir.to_string_lossy().to_string(),
            env_pkg_root: format!("/acacia/{architecture}/{}/{}/root", self.name, self.version),
        };

        // The 'prepare' build step for the formula
        if let Some(cmd) = &self.prepare {
            res.push(PackageBuildStep {
                name: "prepare".to_string(),
                workdir: formula_mountpoint.to_string_lossy().to_string(),

                command: cmd.clone(),

                env_vars: env_vars.clone(),
            })
        }

        // The 'build' build step for the formula
        if let Some(cmd) = &self.build {
            res.push(PackageBuildStep {
                name: "build".to_string(),
                workdir: formula_mountpoint.to_string_lossy().to_string(),

                command: cmd.clone(),

                env_vars: env_vars.clone(),
            })
        }

        // The 'check' build step for the formula
        if let Some(cmd) = &self.check {
            res.push(PackageBuildStep {
                name: "check".to_string(),
                workdir: formula_mountpoint.to_string_lossy().to_string(),

                command: cmd.clone(),

                env_vars: env_vars.clone(),
            })
        }

        // The 'package' build step for the formula
        if let Some(cmd) = &self.package {
            res.push(PackageBuildStep {
                name: "package".to_string(),
                workdir: formula_mountpoint.to_string_lossy().to_string(),

                command: cmd.clone(),

                env_vars: env_vars.clone(),
            })
        }

        Some(res)
    }
}

impl NamedPackage for FormulaPackage {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl VersionedPackage for FormulaPackage {
    fn get_version(&self) -> &str {
        &self.version
    }
}

impl NameVersionPackage for FormulaPackage {}

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

/// The environment variables for a package build step
#[derive(Clone)]
pub struct PackageEnvVars {
    /// `PKG_NAME`
    env_pkg_name: String,
    /// `PKG_VERSION`
    env_pkg_version: String,
    /// `PKG_ARCH`
    env_pkg_arch: String,
    /// `PKG_INSTALL_DIR`
    env_pkg_install_dir: String,
    /// `PKG_ROOT`
    env_pkg_root: String,
}

/// A build step for a package
pub struct PackageBuildStep {
    /// The name of the build step
    name: String,
    /// The working directory
    workdir: String,

    /// The command to execute
    command: String,

    /// The environment variables to provide
    env_vars: PackageEnvVars,
}

impl EnvironmentExecutable for PackageBuildStep {
    fn get_env_variables(&self) -> HashMap<String, String> {
        let mut res = HashMap::new();

        res.insert("PKG_NAME".to_owned(), self.env_vars.env_pkg_name.clone());
        res.insert(
            "PKG_VERSION".to_owned(),
            self.env_vars.env_pkg_version.clone(),
        );
        res.insert("PKG_ARCH".to_owned(), self.env_vars.env_pkg_arch.clone());
        res.insert(
            "PKG_INSTALL_DIR".to_owned(),
            self.env_vars.env_pkg_install_dir.clone(),
        );
        res.insert("PKG_ROOT".to_owned(), self.env_vars.env_pkg_root.clone());

        res
    }

    fn get_command(&self) -> std::ffi::OsString {
        self.command.clone().into()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_workdir(&self) -> std::ffi::OsString {
        self.workdir.clone().into()
    }
}
