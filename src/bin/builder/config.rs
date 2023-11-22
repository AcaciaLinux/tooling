use std::path::PathBuf;

use clap::Parser;

const DEFAULT_WORKDIR: &str = "./";
const DEFAULT_ACACIA_DIR: &str = "/acacia";

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

    /// The formula to build
    pub formula: PathBuf,
}

impl BuilderConfig {
    /// Constructs a builder cofiguration assuming some sane defaults
    /// # Arguments:
    /// * `toolchain` - The mount-internal path to the toolchain binaries (/bin) to prepend to the PATH variable
    /// * `formula_path` - The path to the formula to build
    #[allow(dead_code)]
    pub fn new(toolchain: PathBuf, formula_path: PathBuf) -> BuilderConfig {
        BuilderConfig {
            toolchain,
            workdir: DEFAULT_WORKDIR.into(),
            acacia_dir: DEFAULT_ACACIA_DIR.into(),
            overlay_dirs: Vec::new(),
            exec: None,
            arch: None,
            formula: formula_path,
        }
    }

    /// Returns the directory for the overlayfs inside the workdir: `<workdir>/overlay`
    pub fn get_overlay_dir(&self) -> PathBuf {
        self.workdir.join("overlay")
    }

    /// Returns the directory for the overlayfs work dir inside the workdir: `<workdir>/overlay/work`
    pub fn get_overlay_work_dir(&self) -> PathBuf {
        self.get_overlay_dir().join("work")
    }

    /// Returns the directory for the overlayfs upper dir inside the workdir: `<workdir>/overlay/work`
    pub fn get_overlay_upper_dir(&self) -> PathBuf {
        self.get_overlay_dir().join("upper")
    }

    /// Returns the directory for the overlayfs merged dir to chroot into inside the workdir: `<workdir>/overlay/work`
    pub fn get_overlay_merged_dir(&self) -> PathBuf {
        self.get_overlay_dir().join("merged")
    }

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
