//! The builder tool to build packages using the build pipeline
use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use log::{debug, info};

use crate::{
    env::{BuildEnvironment, Environment, EnvironmentExecutable},
    error::{Error, ErrorExt, ErrorType, Throwable},
    files::formula::FormulaFile,
    package::{
        BuiltPackage, CorePackage, InstalledPackage, InstalledPackageIndex, PackageIndexProvider,
    },
    util::{
        mount::{BindMount, OverlayMount},
        parse::parse_toml,
        signal::SignalDispatcher,
    },
};

/// A template struct that can be used to instantiate `Builder`s
pub struct BuilderTemplate<'a> {
    /// The directory to expect the toolchain binaries at (gets appended with `/bin` for the PATH variable)
    pub toolchain: PathBuf,

    /// The directory the builder works in and stores temporary files
    pub workdir: PathBuf,

    /// The directory to use as the `dist` directory to get read-only bind mounted into the build root
    /// and to search for package dependencies in
    pub dist_dir: PathBuf,

    /// Additional directories to overlay on top of the toolchain and the packages
    pub overlay_dirs: Vec<PathBuf>,

    /// The architecture to build for
    pub arch: String,

    /// The path to the formula to be built
    pub formula_path: PathBuf,

    pub package_index_provider: &'a dyn PackageIndexProvider,
}

/// The `Builder` tool - the configuration struct
pub struct Builder {
    /// The directory to expect the toolchain binaries at (gets appended with `/bin` for the PATH variable)
    pub toolchain: PathBuf,

    /// The directory the builder works in and stores temporary files
    pub workdir: PathBuf,

    /// The directory to use as the `dist` directory to get read-only bind mounted into the build root
    /// and to search for package dependencies in
    pub dist_dir: PathBuf,

    /// Additional directories to overlay on top of the toolchain and the packages
    pub overlay_dirs: Vec<PathBuf>,

    /// Construct the build root, chroot into it and execute this command instead of the build steps
    //pub exec: Option<String>,

    /// The architecture to build for
    pub arch: String,

    /// The package index to consult for information about available packages
    pub target_dependency_index: InstalledPackageIndex,

    /// The path to the formula file
    formula_path: PathBuf,

    /// The formula to build
    pub formula: FormulaFile,
}

/// Creates an installed package index including only the requested packages
/// # Arguments
/// * `packages` - The packages to search for
/// * `provider` - A provider for the searched packages
/// * `dist_dir` - The directory to search for packages
fn find_packages(
    packages: &[String],
    provider: &dyn PackageIndexProvider,
    dist_dir: &Path,
) -> Result<InstalledPackageIndex, Error> {
    let mut res = InstalledPackageIndex::default();

    for package in packages {
        if let Some(package) = provider.find_package(package) {
            let path = package.get_path(dist_dir);

            let package = InstalledPackage::parse_from_index(package, dist_dir)?;

            debug!(
                "Found package {} at {}",
                package.get_full_name(),
                path.to_string_lossy()
            );

            res.push(package);
        } else {
            return Err(BuilderError::DependencyNotFound {
                name: package.clone(),
            }
            .throw("Finding installed packages".to_owned()));
        }
    }

    Ok(res)
}

impl Builder {
    /// Create a Builder from the provided template
    ///
    /// This will parse the formula file and resolve all dependencies to prepare the Builder struct for its job
    pub fn from_template(template: BuilderTemplate) -> Result<Self, Error> {
        // Parse the formula file
        let formula: FormulaFile = parse_toml(&template.formula_path).e_context(|| {
            format!(
                "Parsing formula file {}",
                &template.formula_path.to_string_lossy()
            )
        })?;
        let formula_path = template.formula_path;

        let dependencies: Vec<String> =
            if let Some(dependencies) = &formula.package.target_dependencies {
                dependencies.clone()
            } else {
                Vec::new()
            };

        // Ensure the target dependencies
        let target_dependency_index = find_packages(
            &dependencies,
            template.package_index_provider,
            &template.dist_dir,
        )?;

        Ok(Self {
            toolchain: template.toolchain,
            workdir: template.workdir,
            dist_dir: template.dist_dir,
            overlay_dirs: template.overlay_dirs,
            arch: template.arch,
            target_dependency_index,
            formula_path,
            formula,
        })
    }

    /// Executes the build job by calling all the build steps in sequence
    /// # Arguments
    /// * `signal_dispatcher` - The signal dispatcher to use for handing signals
    pub fn build(&self, signal_dispatcher: &SignalDispatcher) -> Result<BuiltPackage, Error> {
        let context = || {
            format!(
                "Building package {}",
                self.formula.package.get_full_name(&self.arch)
            )
        };
        info!(
            "Building package {}",
            self.formula.package.get_full_name(&self.arch)
        );

        let env = self.create_env().e_context(context)?;

        // Try to retrieve the build steps from the formula
        let steps = match self.formula.package.get_buildsteps(
            Path::new("/formula"),
            &self.arch,
            &PathBuf::from("/")
                .join(self.formula.package.get_full_name(&self.arch))
                .join("data"),
        ) {
            Some(steps) => steps,
            None => {
                return Err(BuilderError::UnsupportedArch {
                    arch: self.arch.to_owned(),
                    available_archs: self.formula.package.get_architectures(),
                }
                .throw(format!(
                    "Constructing build steps for package {}",
                    self.formula.package.get_full_name(&self.arch)
                )));
            }
        };

        // Try running each step
        for step in steps {
            let status = env.execute(&step, signal_dispatcher)?;
            if !status.success() {
                return Err(BuilderError::CommandFailed { status }
                    .throw(format!("Running build step '{}'", step.get_name())));
            }
        }

        BuiltPackage::from_formula(
            self.formula.package.clone(),
            self.arch.clone(),
            &self.destdir_outer_path(),
        )
    }

    /// Run a custom executable using a provided environment
    /// # Argumets
    /// * `environment` - The environment to use for running the executable
    /// * `executable` - The executable to run
    /// *  `signal_dispatcher` - The signal dispatcher to use
    pub fn run_custom(
        &self,
        environment: &BuildEnvironment,
        executable: &dyn EnvironmentExecutable,
        signal_dispatcher: &SignalDispatcher,
    ) -> Result<std::process::ExitStatus, Error> {
        let status = environment
            .execute(executable, signal_dispatcher)
            .e_context(|| "Running custom executable".to_owned())?;

        Ok(status)
    }

    /// Creates an environment using this builder's information
    ///
    /// This will mount the overlayfs, bind mounts and the vkfs mount
    pub fn create_env(&self) -> Result<BuildEnvironment, Error> {
        let formula_parent = self
            .formula_path
            .parent()
            .expect("Parent directoriy for formula");

        // The overlay mount for the root filesystem
        let mut lower_dirs = self.overlay_dirs.clone();
        // Append the `root` directories of all the `target_dependencies`
        lower_dirs.append(
            &mut self
                .target_dependency_index
                .inner()
                .iter()
                .map(|p| p.get_path(&self.dist_dir).join("root"))
                .collect(),
        );
        let root_mount = OverlayMount::new(
            lower_dirs,
            self.get_root_overlay_dir().join("work"),
            self.get_root_overlay_dir().join("upper"),
            self.get_overlay_dir().join("merged"),
        )
        .e_context(|| "Mounting root overlayfs".to_owned())?;

        // The overlay mount for the formula's parent directory
        let formula_mount = OverlayMount::new(
            vec![formula_parent.to_owned()],
            self.get_formula_overlay_dir().join("work"),
            self.get_formula_overlay_dir().join("upper"),
            self.get_overlay_dir().join("merged").join("formula"),
        )
        .e_context(|| "Mounting formula overlayfs".to_owned())?;

        // The mount for the installation directory
        let install_mount = BindMount::new(
            &self.destdir_outer_path(),
            &self.destdir_inner_path(),
            false,
        )?;

        let mut env = BuildEnvironment::new(Box::new(root_mount), self.toolchain.clone())
            .e_context(|| "Creating build environment".to_owned())?;

        env.add_mount(Box::new(formula_mount));
        env.add_mount(Box::new(install_mount));

        Ok(env)
    }

    /// The directory to house all overlay directories
    fn get_overlay_dir(&self) -> PathBuf {
        self.workdir.join("overlay")
    }

    /// The directory to house the overlay directories of the root mount
    fn get_root_overlay_dir(&self) -> PathBuf {
        self.get_overlay_dir().join("root")
    }

    /// The directory to house the overlay directories of the formula mount
    fn get_formula_overlay_dir(&self) -> PathBuf {
        self.get_overlay_dir().join("formula")
    }

    /// The package installation directory from outside the chroot
    fn destdir_outer_path(&self) -> PathBuf {
        self.workdir
            .join(self.formula.package.get_full_name(&self.arch))
    }

    /// The package installation directory from inside the chroot
    fn destdir_inner_path(&self) -> PathBuf {
        self.get_overlay_dir()
            .join("merged")
            .join(self.formula.package.get_full_name(&self.arch))
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
    fn e_context<F: Fn() -> String>(self, context: F) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new_context(ErrorType::Builder(e), context())),
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