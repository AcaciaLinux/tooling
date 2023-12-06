use std::path::PathBuf;

use clap::Parser;

const DEFAULT_WORKDIR: &str = "./work";
const DEFAULT_ACACIA_DIR: &str = "/acacia";
const DEFAULT_PACKAGE_INDEX: &str = "/acacia/packages.toml";

/// Build AcaciaLinux packages
#[derive(Parser)]
#[command(author = "Max Kofler", name = "builder", version, about = include_str!("about.txt"), long_about = include_str!("long_about.txt"))]
pub struct BuilderConfig {
    /// The directory to expect the toolchain binaries at (appended with '/bin' for 'PATH')
    #[arg(long)]
    pub toolchain: PathBuf,

    /// The directory the builder works in and stores temporary files
    #[arg(long, default_value = DEFAULT_WORKDIR)]
    pub workdir: PathBuf,

    /// The directory to search for package dependencies
    #[arg(long, default_value = DEFAULT_ACACIA_DIR)]
    pub acacia_dir: PathBuf,

    /// Additional directories to overlay on top of the toolchain and the packages
    #[arg(long)]
    pub overlay_dirs: Vec<PathBuf>,

    #[arg(long, short)]
    /// Chroot and execute a custom command
    pub exec: Option<String>,

    #[arg(long)]
    /// The architecture to build for
    pub arch: Option<String>,

    /// The path to the package index for searching packages
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
