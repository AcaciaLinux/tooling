use std::path::PathBuf;

use clap::Parser;

const DEFAULT_WORKDIR: &str = "./work";
const DEFAULT_DIST_DIR: &str = "/acacia";
const DEFAULT_OUTPUT_DIR: &str = "./";

/// Build AcaciaLinux packages
#[derive(Parser)]
#[command(author = "Max Kofler", name = "branch", version, about = include_str!("about.txt"), long_about = include_str!("long_about.txt"))]
pub struct BuilderConfig {
    /// The loglevel to operate on (0 = info, 1 = debug, * = trace)
    #[arg(long = "loglevel", short = 'v', default_value_t = 0)]
    pub loglevel: u8,

    /// The directory to expect the toolchain binaries at (appended with '/bin' for 'PATH')
    #[arg(long)]
    pub toolchain: PathBuf,

    /// The directory the builder works in and stores temporary files
    #[arg(long, default_value = DEFAULT_WORKDIR)]
    pub workdir: PathBuf,

    /// The directory to search for package dependencies
    #[arg(long, default_value = DEFAULT_DIST_DIR)]
    pub dist_dir: PathBuf,

    /// Additional directories to overlay on top of the toolchain and the packages
    #[arg(long)]
    pub overlay_dirs: Vec<PathBuf>,

    #[arg(long, short)]
    /// Chroot and execute a custom command
    pub exec: Option<String>,

    #[arg(long)]
    /// The architecture to build for
    pub arch: Option<String>,

    #[arg(long, default_value = DEFAULT_OUTPUT_DIR)]
    /// The directory to put built packages into
    pub output_dir: PathBuf,

    #[arg(long, default_value_t = false)]
    /// Skip validation actions, leaving the built package in an unpatched state
    pub skip_validation: bool,

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
