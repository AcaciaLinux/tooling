use std::{collections::HashMap, path::PathBuf, process::ExitStatus};

mod workdir;
use log::{debug, info};
pub use workdir::*;

use crate::{
    env::{executable::CustomExecutable, BuildEnvironment, Environment},
    error::{Error, ErrorExt, ErrorType, Throwable},
    model::{Formula, Home, ObjectDB},
    util::{
        fs::{self, create_dir_all, PathUtil},
        mount::OverlayMount,
        signal::SignalDispatcher,
    },
};

pub struct Builder<'a> {
    workdir: BuilderWorkdir,
    formula: Formula,
    object_db: &'a ObjectDB,
}

impl<'a> Builder<'a> {
    /// Creates a new instance of the builder tool
    ///
    /// # Arguments
    /// * `home` - The home to use for storing temporary files
    /// * `formula` - The formula to use as a source when building
    /// * `object_db` - The object database to use for retrieving and storing objects
    /// # Returns
    /// An instance of the builder tool to be used for building the formula
    pub fn new(home: &Home, formula: Formula, object_db: &'a ObjectDB) -> Result<Self, Error> {
        Ok(Self {
            workdir: BuilderWorkdir::new(home)?,
            formula,
            object_db,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let formula_oid = self.formula.oid();
        let formula_inner_path = PathBuf::from(formula_oid.to_string());

        let formula_root = self.workdir.get_formula_dir().join(&formula_inner_path);

        // Extract the source files from the object database
        // and deploy them to 'formula_root'
        debug!("Extracting sources from object database...");
        for (path, oid) in self.formula.files.iter() {
            let full_path = formula_root.join(path.make_relative());

            if let Some(parent) = full_path.parent() {
                create_dir_all(parent)?;
            }

            self.object_db
                .read_to_file(oid, &full_path)
                .e_context(|| format!("Extracting object {} to {}", oid, path.str_lossy(),))?;
        }

        // Use a separate scope for all functions that
        // need an active environment. This makes sure
        // the environment is dropped and removed once
        // it is no longer needed
        {
            let lower_dirs = vec![self.workdir.get_formula_dir()];

            let overlay_mount = OverlayMount::new(
                lower_dirs,
                self.workdir.get_overlay_dir_work(),
                self.workdir.get_overlay_dir_upper(),
                self.workdir.get_overlay_dir_merged(),
            )
            .ctx(|| "Creating overlay mount")?;

            let environment = BuildEnvironment::new(Box::new(overlay_mount))
                .ctx(|| "Creating build environment")?;

            let mut envs = HashMap::new();
            envs.insert("PKG_NAME".to_owned(), self.formula.name.clone());
            envs.insert("PKG_VERSION".to_owned(), self.formula.version.clone());

            // For now, we execute 'sh' before implementing the build commands
            let executable = CustomExecutable::new("sh".to_owned(), formula_inner_path, envs);
            let dispatcher = SignalDispatcher::default();

            info!("Build environment is ready - entering chroot");

            environment.execute(&executable, &dispatcher)?;
        }

        debug!("Exited from chroot, cleaning up...");
        fs::remove_dir_all(&formula_root)?;

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
