use std::path::PathBuf;

use log::info;

use crate::{
    cache::download::DownloadCache,
    env::executable::BuildStep,
    error::{Error, ErrorExt},
    files::formula::FormulaFile,
    tools::builder::{BuilderError, BuilderWorkdir},
    util::{self, architecture::Architecture},
};

use super::{CorePackage, DescribedPackage, NameVersionPackage, NamedPackage, VersionedPackage};

/// A package that can be built in a `BuildEnvironment`
#[derive(Debug)]
pub struct BuildablePackage<'a> {
    /// The formula to wrap and build
    formula: FormulaFile,
    /// The architecture to build the formula for
    architecture: Architecture,
    /// The working directory to use for building
    workdir: &'a BuilderWorkdir,
}

impl<'a> BuildablePackage<'a> {
    /// Constructs a buildable package from a formula file, fetching the sources
    /// and preparing it for building
    ///
    /// This function will not succeed unless the resulting package can be built
    /// # Arguments
    /// * `formula` - The formula to wrap in this package
    /// * `architecture` - The architecture the package should be buildable for
    pub fn from_formula(
        formula: FormulaFile,
        architecture: Architecture,
        workdir: &'a BuilderWorkdir,
        cache: &DownloadCache,
    ) -> Result<Self, Error> {
        // First, make sure we can even build the formula for the architecture
        Self::ensure_buildable(&formula, &architecture)?;

        // Create the package
        let pkg = Self {
            formula: formula.clone(),
            architecture,
            workdir,
        };

        // Ensure sources are present
        pkg.fetch_and_extract_sources(cache)?;

        Ok(pkg)
    }

    /// Returns the working directory for this buildable package
    pub fn get_workdir(&self) -> &BuilderWorkdir {
        self.workdir
    }

    /// Returns the formula this package will use for building
    pub fn get_formula(&self) -> &FormulaFile {
        &self.formula
    }

    /// Returns the UUID for this package
    pub fn get_id(&self) -> &str {
        self.workdir.get_id()
    }

    /// Returns the Architecture for this package
    pub fn get_arch(&self) -> &Architecture {
        &self.architecture
    }

    /// Returns the build steps for this package to be executed
    /// in the order they are returned from this function
    pub fn get_buildsteps(&self) -> Vec<BuildStep> {
        let mut res = Vec::new();

        if let Some(step) = &self.formula.package.prepare {
            res.push(self.create_buildstep("Prepare".to_owned(), step.to_owned()))
        }

        if let Some(step) = &self.formula.package.build {
            res.push(self.create_buildstep("Build".to_owned(), step.to_owned()))
        }

        if let Some(step) = &self.formula.package.check {
            res.push(self.create_buildstep("Check".to_owned(), step.to_owned()))
        }

        if let Some(step) = &self.formula.package.package {
            res.push(self.create_buildstep("Package".to_owned(), step.to_owned()))
        }

        res
    }

    /// Creates a build step with the information from this package
    /// # Arguments
    /// * `name` - The name for the build step
    /// * `command` - The command to execute for this buildstep
    fn create_buildstep(&self, name: String, command: String) -> BuildStep {
        BuildStep {
            name,
            pkg_info: self.get_info(),
            arch: self.architecture.clone(),
            command,
            workdir: PathBuf::from("/"),
            install_dir: self.workdir.get_install_dir_inner(),
        }
    }
}

impl<'a> BuildablePackage<'a> {
    /// Makes sure the formula is buildable for `architecture`
    /// # Arguments
    /// * `formula` - The formula to validate
    /// * `architecture` - The architecture to check build support for
    /// # Returns
    /// An error if the build cannot take place
    fn ensure_buildable(formula: &FormulaFile, architecture: &Architecture) -> Result<(), Error> {
        // Check if at least one architecture can run on the host and use that
        if let Some(archs) = &formula.package.arch {
            let mut err = false;

            for a in archs {
                err |= !architecture.can_host(a);
            }

            if err {
                return Err(BuilderError::UnsupportedArch {
                    arch: architecture.arch.clone(),
                    available_archs: archs.iter().map(|a| a.arch.clone()).collect(),
                })
                .e_context(|| {
                    format!(
                        "Checking valid host architectures for {}-{}",
                        formula.package.name, formula.package.version
                    )
                });
            }
        }

        Ok(())
    }

    /// Fetches and extracts sources
    /// # Arguments
    /// * `cache` - The download cache to use for caching downloads
    fn fetch_and_extract_sources(&self, cache: &DownloadCache) -> Result<(), Error> {
        // Fetch and extract sources
        if let Some(sources) = &self.formula.package.sources {
            for src in sources {
                let url = src.get_url(self);
                let dest = src.get_dest(self);

                let context = || format!("Fetching source '{url}' to '{dest}'",);

                let formula_dir = self.workdir.get_formula_dir();
                let full_dest_dir = formula_dir.join(&dest);

                util::fs::create_dir_all(&formula_dir).e_context(context)?;

                cache
                    .download(
                        &url,
                        &full_dest_dir,
                        &format!("Fetching '{url}' to '{dest}'"),
                        true,
                    )
                    .e_context(context)?;

                if src.extract {
                    info!("Extracting {}...", dest);
                    util::archive::extract_infer(&full_dest_dir, &formula_dir).e_context(context)?
                }
            }
        }

        Ok(())
    }
}

impl<'a> NamedPackage for BuildablePackage<'a> {
    fn get_name(&self) -> &str {
        &self.formula.package.name
    }
}

impl<'a> VersionedPackage for BuildablePackage<'a> {
    fn get_version(&self) -> &str {
        &self.formula.package.version
    }

    fn get_pkgver(&self) -> u32 {
        self.formula.package.pkgver
    }

    fn get_id(&self) -> &str {
        self.workdir.get_id()
    }
}

impl<'a> NameVersionPackage for BuildablePackage<'a> {}

impl<'a> DescribedPackage for BuildablePackage<'a> {
    fn get_description(&self) -> &str {
        &self.formula.package.description
    }
}

impl<'a> CorePackage for BuildablePackage<'a> {}
