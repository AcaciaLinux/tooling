use std::path::PathBuf;

use clap::Parser;

const DEFAULT_WORKDIR: &str = "./work";
const DEFAULT_ACACIA_DIR: &str = "/acacia";
const DEFAULT_PACKAGE_INDEX: &str = "/acacia/packages.toml";

#[derive(Parser)]
pub struct BuilderConfig {
    /// The directory to expect the toolchain binaries at (gets appended with `/bin` for the PATH variable)
    #[arg(long)]
    pub toolchain: PathBuf,

    /// The directory the builder works in and stores temporary files
    #[arg(long, default_value = DEFAULT_WORKDIR)]
    pub workdir: PathBuf,

    /// The directory to use as the `/acacia` directory to get read-only bind mounted into the build root
    /// and to search for package dependencies in
    #[arg(long, default_value = DEFAULT_ACACIA_DIR)]
    pub acacia_dir: PathBuf,

    /// Additional directories to overlay on top of the toolchain and the packages
    #[arg(long)]
    pub overlay_dirs: Vec<PathBuf>,

    #[arg(long, short)]
    /// Construct the build root, chroot into it and execute this command instead of the build steps
    pub exec: Option<String>,

    #[arg(long)]
    /// The architecture to build for
    pub arch: Option<String>,

    /// The path to the package index to use for inferring package availability
    #[arg(long, default_value = DEFAULT_PACKAGE_INDEX)]
    pub package_index: PathBuf,

    /// The formula to build
    pub formula: PathBuf,
}

impl BuilderConfig {
    /// Returns the architecture the builder should build for, using the `uname` crate
    /// or the overridden value (`--arch`)
    pub fn get_arch(&self) -> String {
        match &self.arch {
            Some(a) => a.clone(),
            None => {
                uname::uname()
                    .expect("Unable to read machine information")
                    .machine
            }
        }
    }
}
