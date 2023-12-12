//! The builder tool to build packages using the build pipeline
use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use log::info;

use crate::{
    cache::download::DownloadCache,
    env::{BuildEnvironment, Environment, EnvironmentExecutable},
    error::{Error, ErrorExt, ErrorType, Throwable},
    files::formula::FormulaFile,
    package::{BuiltPackage, CorePackage, InstalledPackageIndex, PackageInfo},
    util::{
        archive,
        mount::{BindMount, OverlayMount},
        parse::{parse_toml, versionstring::VersionString},
        signal::SignalDispatcher,
    },
    validators::{indexed_package::FileValidationResult, ValidationInput},
    ANY_ARCH,
};

/// A template struct that can be used to instantiate `Builder`s
pub struct BuilderTemplate {
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

    /// The architecture to build for
    pub arch: String,

    /// The package index to consult for information about available packages
    pub target_dependency_index: InstalledPackageIndex,

    /// The path to the formula file
    formula_path: PathBuf,

    /// The formula to build
    pub formula: FormulaFile,

    /// The build-id for the current builder instance
    build_id: String,

    /// A cache for source files
    source_cache: DownloadCache,
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

        // If the architecture of the formula is 'Any', use it instead of the target architecture
        let arch = match formula.package.arch.clone() {
            None => ANY_ARCH.to_owned(),
            Some(_) => template.arch,
        };

        let formula_path = template.formula_path;

        let dependencies: Vec<VersionString> =
            if let Some(dependencies) = &formula.package.target_dependencies {
                dependencies.clone()
            } else {
                Vec::new()
            };

        // Ensure the target dependencies
        let target_dependency_index = InstalledPackageIndex::from_package_list(
            &dependencies,
            arch.clone(),
            &template.dist_dir,
        )?;

        let source_cache = DownloadCache::new(template.workdir.join("source_cache"))?;

        Ok(Self {
            toolchain: template.toolchain,
            workdir: template.workdir,
            dist_dir: template.dist_dir,
            overlay_dirs: template.overlay_dirs,
            arch,
            target_dependency_index,
            formula_path,
            formula,
            build_id: uuid::Uuid::new_v4().to_string(),
            source_cache,
        })
    }

    /// Executes the build job by calling all the build steps in sequence and validates the package
    ///
    /// This will run the whole chain of build events to produce a `BuildPackage`
    /// # Arguments
    /// * `signal_dispatcher` - The signal dispatcher to use for handing signals
    /// # Returns
    /// The `BuiltPackage` and a vector of validation results for the package, else a fatal error
    pub fn build(
        &self,
        signal_dispatcher: &SignalDispatcher,
    ) -> Result<(BuiltPackage, Vec<FileValidationResult>), Error> {
        let context = || {
            format!(
                "Building package {}",
                self.formula.package.get_full_name(&self.arch)
            )
        };
        let package_info = PackageInfo {
            name: self.formula.package.name.clone(),
            version: self.formula.package.version.clone(),
            pkgver: self.formula.package.pkgver,
            arch: self.arch.clone(),
        };
        info!(
            "Building package {} (build-id: {})",
            package_info.get_full_name(),
            self.build_id
        );

        let env = self.create_env().e_context(context)?;

        info!(
            "Starting to build '{}'",
            self.formula.package.get_full_name(&self.arch)
        );

        // Try to retrieve the build steps from the formula
        let steps = match self.formula.package.get_buildsteps(
            Path::new("/formula"),
            &self.arch,
            &PathBuf::from("/").join(&self.build_id).join("data"),
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
            info!("Running build step '{}'", step.get_name());
            let status = env.execute(&step, signal_dispatcher)?;
            if !status.success() {
                return Err(BuilderError::CommandFailed { status }
                    .throw(format!("Running build step '{}'", step.get_name())));
            }
            info!("Build step '{}' exited with {}", step.get_name(), status);
        }

        let validation_input = ValidationInput {
            package_index: &self.target_dependency_index,
        };

        BuiltPackage::from_formula_validate(
            self.formula.clone(),
            self.arch.clone(),
            &self.destdir_outer_path(),
            &validation_input,
            self.build_id.clone(),
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
        let package_info = PackageInfo {
            name: self.formula.package.name.clone(),
            version: self.formula.package.version.clone(),
            pkgver: self.formula.package.pkgver,
            arch: self.arch.clone(),
        };

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

        // Download the sources and extract them if so desired
        if let Some(sources) = &self.formula.package.sources {
            for source in sources {
                let url = source.get_url(&package_info);
                let dest = source.get_dest(&package_info);

                let formula_dir = &self.get_overlay_dir().join("merged").join("formula");
                let dest_path = formula_dir.join(&dest);

                self.source_cache.download(
                    &url,
                    &dest_path,
                    &format!("Downloading source '{}' to '{}'", url, dest),
                    true,
                )?;

                if source.extract {
                    info!(
                        "Extracting {} to {}",
                        dest_path.to_string_lossy(),
                        formula_dir.to_string_lossy()
                    );

                    archive::extract_infer(&dest_path, formula_dir)
                        .e_context(|| "Creating build environment".to_owned())?;
                }
            }
        }

        Ok(env)
    }

    /// The directory to house all overlay directories
    fn get_overlay_dir(&self) -> PathBuf {
        self.workdir
            .join("builds")
            .join(&self.build_id)
            .join("overlay")
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
            .join("builds")
            .join(&self.build_id)
            .join("package-archive")
    }

    /// The package installation directory from inside the chroot
    fn destdir_inner_path(&self) -> PathBuf {
        self.get_overlay_dir().join("merged").join(&self.build_id)
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
