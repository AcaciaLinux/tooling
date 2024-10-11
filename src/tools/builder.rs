use std::{collections::HashMap, path::PathBuf, process::ExitStatus};

mod workdir;
use log::{debug, error, info, warn};
pub use workdir::*;

use crate::{
    dist_dir,
    env::{BuildEnvironment, Environment, EnvironmentExecutable},
    error::{Error, ErrorExt, ErrorType, Throwable},
    model::{Formula, Home, Object, ObjectCompression, ObjectDB},
    tools::indexer::Indexer,
    util::{
        fs::{self, create_dir_all, PathUtil},
        mount::OverlayMount,
        signal::SignalDispatcher,
    },
};

pub struct Builder<'a> {
    workdir: BuilderWorkdir,
    formula: Formula,
    object_db: &'a mut ObjectDB,
    env_vars: HashMap<String, String>,
    lower_dirs: Vec<PathBuf>,
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
    pub fn new(home: &Home, formula: Formula, object_db: &'a mut ObjectDB) -> Result<Self, Error> {
        Ok(Self {
            workdir: BuilderWorkdir::new(home)?,
            formula,
            object_db,
            env_vars: HashMap::new(),
            lower_dirs: Vec::new(),
        })
    }

    /// Adds a new environment variable to the default variables
    /// # Arguments
    /// * `key` - The key (name) of the variable
    /// * `value` - The value of the variable
    pub fn add_env_var(&mut self, key: String, value: String) {
        self.env_vars.insert(key, value);
    }

    /// Adds a hashmap of environment variables to the default variables
    /// # Arguments
    /// * `envs` - The hashmap to extend the internal with
    pub fn add_env_vars(&mut self, envs: HashMap<String, String>) {
        self.env_vars.extend(envs);
    }

    /// Adds an additional lower directory to the list
    /// of mounts
    /// # Arguments
    /// * `dir` - The directory to mount
    pub fn add_lower_dir(&mut self, dir: PathBuf) {
        self.lower_dirs.push(dir);
    }

    /// Adds additional lower directories to the list
    /// of mounts
    /// # Arguments
    /// * `dirs` - The directories to mount
    pub fn add_lower_dirs(&mut self, dirs: Vec<PathBuf>) {
        self.lower_dirs.extend(dirs);
    }

    pub fn run(
        &mut self,
        signal_dispatcher: &SignalDispatcher,
        compression: ObjectCompression,
        skip_duplicates: bool,
    ) -> Result<Object, Error> {
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

        let buildsteps = self.formula.get_buildsteps(
            formula_inner_path.clone(),
            self.workdir.get_install_dir_inner(),
        );

        let tainted = !(self.env_vars.is_empty() & self.lower_dirs.is_empty());

        // Use a separate scope for all functions that
        // need an active environment. This makes sure
        // the environment is dropped and removed once
        // it is no longer needed
        {
            let mut lower_dirs = vec![self.workdir.get_formula_dir()];
            lower_dirs.extend(self.lower_dirs.clone());

            let overlay_mount = OverlayMount::new(
                lower_dirs,
                self.workdir.get_overlay_dir_work(),
                self.workdir.get_overlay_dir_upper(),
                self.workdir.get_overlay_dir_merged(),
            )
            .ctx(|| "Creating overlay mount")?;

            let mut environment = BuildEnvironment::new(Box::new(overlay_mount))
                .ctx(|| "Creating build environment")?;

            environment.add_envs(self.env_vars.clone());

            info!("Build environment is ready - executing buildsteps");

            for buildstep in buildsteps {
                info!("Executing step '{}'...", buildstep.get_name());

                let status = environment
                    .execute(&buildstep, signal_dispatcher)
                    .ctx(|| format!("Executing '{}' step", buildstep.get_name()))?;

                if !status.success() {
                    error!(
                        "Build step '{}' failed with exit code {}",
                        buildstep.get_name(),
                        status
                    );
                    return Err(Error::new_context(
                        BuilderError::CommandFailed { status }.into(),
                        format!("Executing build step '{}'", buildstep.get_name()),
                    ));
                }

                info!("Executed step '{}'...", buildstep.get_name());
            }
        }

        debug!("Exited from chroot, cleaning up...");
        fs::remove_dir_all(&formula_root)?;

        let indexer = Indexer::new(
            self.workdir
                .get_install_dir_upper()
                .join(dist_dir())
                .join("pkg")
                .join(formula_oid.to_string())
                .join("root"),
        );

        let index = indexer
            .run(true, &mut self.object_db, compression, skip_duplicates)
            .ctx(|| "Indexing finished package")?;

        let indexfile = index.to_index_file();

        let object = indexfile.insert(&mut self.object_db, compression)?;

        if tainted {
            warn!("This build is TAINTED!");
        }

        Ok(object)
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

impl From<BuilderError> for ErrorType {
    fn from(value: BuilderError) -> Self {
        ErrorType::Builder(value)
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
