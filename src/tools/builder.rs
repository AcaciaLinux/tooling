use std::{collections::HashMap, path::PathBuf, process::ExitStatus};

mod workdir;
use log::{info, warn};
pub use workdir::*;

use crate::{
    env::{BuildEnvironment, Environment, EnvironmentExecutable},
    error::{Error, ErrorExt, ErrorType, Throwable},
    model::{BuildStep, BuildStepType, Formula, Home, ObjectDB, Tree},
    util::{fs::PathUtil, mount::OverlayMount, signal::SignalDispatcher, ODBUnpackable},
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

    fn get_overlay_src_dir(&self) -> PathBuf {
        self.get_overlay_dir().join("source")
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
        odb: &ObjectDB,
        additional_lowerdirs: Vec<PathBuf>,
        additional_paths: Vec<PathBuf>,
        signal_dispatcher: &SignalDispatcher,
    ) -> Result<(), Error> {
        let tainted = !additional_lowerdirs.is_empty();
        if tainted {
            for dir in &additional_lowerdirs {
                warn!("Build is tainted with directory '{}'", dir.str_lossy());
            }
        }

        // Deploy the formula's tree to later base our overlay fs on
        let src_dir = self.get_overlay_src_dir();
        let mut tree_object = odb.read(&self.formula.tree).ctx(|| "Opening tree object")?;
        let tree = Tree::unpack_from_odb(&mut tree_object, odb).ctx(|| "Reading tree object")?;
        tree.deploy(&src_dir, odb).ctx(|| "Deploying tree")?;

        // Construct the vector of lower directories for the overlay fs
        let mut lower_dirs = vec![src_dir];
        lower_dirs.extend_from_slice(&additional_lowerdirs);

        // Handle additional PATH paths
        let mut path_var = String::new();
        if !additional_paths.is_empty() {
            path_var += ":";
            path_var += &additional_paths
                .iter()
                .map(|p| p.str_lossy())
                .collect::<Vec<String>>()
                .join(":");
        }

        let mut envs = HashMap::new();
        envs.insert("PATH".to_owned(), path_var);

        self.execute_build_step(
            BuildStepType::Prepare,
            &mut lower_dirs,
            signal_dispatcher,
            envs.clone(),
        )?;
        self.execute_build_step(
            BuildStepType::Build,
            &mut lower_dirs,
            signal_dispatcher,
            envs.clone(),
        )?;
        self.execute_build_step(
            BuildStepType::Check,
            &mut lower_dirs,
            signal_dispatcher,
            envs.clone(),
        )?;
        self.execute_build_step(
            BuildStepType::Package,
            &mut lower_dirs,
            signal_dispatcher,
            envs,
        )?;

        Ok(())
    }

    fn execute_build_step(
        &self,
        step: BuildStepType,
        lower_dirs: &mut Vec<PathBuf>,
        signal_dispatcher: &SignalDispatcher,
        environment_variables: HashMap<String, String>,
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
                environment_variables.clone(),
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
                    environment_variables.clone(),
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
        environment_variables: HashMap<String, String>,
    ) -> Result<(), Error> {
        {
            let mount = OverlayMount::new(
                lower_dirs,
                self.get_overlay_workdir(),
                upper_dir,
                self.get_overlay_merged(),
            )?;

            let env = BuildEnvironment::new(Box::new(mount))?;

            env.execute(executable, signal_dispatcher, environment_variables)?;
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
