use std::{collections::HashMap, path::PathBuf, process::ExitStatus};

mod workdir;
use log::{info, warn};
pub use workdir::*;

use crate::{
    env::{BuildEnvironment, Environment, EnvironmentExecutable},
    error::{Error, ErrorExt, ErrorType, Throwable},
    model::{BuildStep, BuildStepType, Formula, Home},
    util::{fs::PathUtil, mount::OverlayMount, signal::SignalDispatcher},
};

pub struct Builder {
    pub formula: Formula,
    root: PathBuf,
}

impl Builder {
    pub fn new(home: &Home, formula: Formula) -> Self {
        Self {
            formula,
            root: home.get_builder_workdir(),
        }
    }

    fn get_overlay_dir(&self) -> PathBuf {
        self.root.join("overlay")
    }

    fn get_overlay_merged(&self) -> PathBuf {
        self.get_overlay_dir().join("merged")
    }

    fn get_overlay_workdir(&self) -> PathBuf {
        self.get_overlay_dir().join("work")
    }

    fn get_overlay_upper(&self) -> PathBuf {
        self.get_overlay_dir().join("upper")
    }

    pub fn build(
        self,
        additional_lowerdirs: Vec<PathBuf>,
        signal_dispatcher: &SignalDispatcher,
    ) -> Result<(), Error> {
        let tainted = !additional_lowerdirs.is_empty();
        if tainted {
            for dir in &additional_lowerdirs {
                warn!("Build is tainted with directory '{}'", dir.str_lossy());
            }
        }

        let mut lower_dirs = Vec::new();
        lower_dirs.extend_from_slice(&additional_lowerdirs);

        let mut envs = HashMap::new();

        envs.insert(
            "PATH".to_owned(),
            "/bin:/usr/bin:/sbin:/usr/sbin".to_owned(),
        );

        self.execute_build_step(BuildStepType::Prepare, &mut lower_dirs, signal_dispatcher)?;
        self.execute_build_step(BuildStepType::Build, &mut lower_dirs, signal_dispatcher)?;
        self.execute_build_step(BuildStepType::Check, &mut lower_dirs, signal_dispatcher)?;
        self.execute_build_step(BuildStepType::Package, &mut lower_dirs, signal_dispatcher)?;

        Ok(())
    }

    fn execute_build_step(
        &self,
        step: BuildStepType,
        lower_dirs: &mut Vec<PathBuf>,
        signal_dispatcher: &SignalDispatcher,
    ) -> Result<(), Error> {
        if let Some(step_cmd) = self.formula.get_build_step(step) {
            info!(
                "Executing formula '{}' build step '{}'...",
                self.formula.name,
                step.string()
            );
            let build_step = BuildStep::new_formula(
                &self.formula,
                step_cmd,
                format!(
                    "Step '{}' for formula '{}'",
                    step.string(),
                    self.formula.name
                ),
            );
            let upper_dir = self.get_overlay_upper().join("formula").join(step.string());

            self.execute(
                &build_step,
                lower_dirs.clone(),
                upper_dir.clone(),
                signal_dispatcher,
            )?;
            lower_dirs.push(upper_dir);
        }

        for (pkg_name, package) in &self.formula.packages {
            if let Some(step_cmd) = package.get_build_step(step) {
                info!(
                    "Executing package '{pkg_name}' build step '{}'...",
                    step.string()
                );
                let upper_dir = self
                    .get_overlay_upper()
                    .join("package")
                    .join(pkg_name)
                    .join(step.string());
                let build_step = BuildStep::new(
                    step_cmd,
                    pkg_name.to_owned(),
                    self.formula.version.to_owned(),
                    format!("Step '{}' for package '{}'", step.string(), pkg_name),
                );

                self.execute(
                    &build_step,
                    lower_dirs.clone(),
                    upper_dir,
                    signal_dispatcher,
                )?;
            }
        }

        Ok(())
    }

    fn execute(
        &self,
        executable: &dyn EnvironmentExecutable,
        lower_dirs: Vec<PathBuf>,
        upper_dir: PathBuf,
        signal_dispatcher: &SignalDispatcher,
    ) -> Result<(), Error> {
        {
            let mount = OverlayMount::new(
                lower_dirs,
                self.get_overlay_workdir(),
                upper_dir,
                self.get_overlay_merged(),
            )?;

            let env = BuildEnvironment::new(Box::new(mount))?;

            env.execute(executable, signal_dispatcher)?;
        }
        Ok(())
    }
}

/// An error that originated from the `Builder` struct
#[derive(Debug)]
pub enum BuilderError {
    /// The builder tried to build a package for a non-supported architecture
    UnsupportedArch {
        arch: String,
        available_archs: Vec<String>,
    },
    /// The builder could not find a dependency for the building process
    DependencyNotFound { name: String },
    /// A subcommand failed and the builder cannot continue working
    CommandFailed { status: ExitStatus },
}

impl<T> ErrorExt<T> for Result<T, BuilderError> {
    fn e_context<S: ToString, F: Fn() -> S>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(
                ErrorType::Builder(e),
                context().to_string(),
            )),
        }
    }
}

impl Throwable for BuilderError {
    fn throw(self, context: String) -> Error {
        Error::new_context(ErrorType::Builder(self), context)
    }
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedArch {
                arch,
                available_archs,
            } => write!(
                f,
                "Unsupported architecture '{}', available architectures: {}",
                arch,
                available_archs.join(", ")
            ),
            Self::DependencyNotFound { name } => {
                write!(f, "Dependency '{name}' not found")
            }
            Self::CommandFailed { status } => {
                write!(f, "Command failed with the following code: {}", status)
            }
        }
    }
}
