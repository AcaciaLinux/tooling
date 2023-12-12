use std::path::PathBuf;

use clap::Parser;

const DEFAULT_PARENT_DIR: &str = "./";

/// Generate formula files from a simple set of questions
#[derive(Parser)]
#[command(author = "Max Kofler", name = "formulagen", version, about, long_about)]
pub struct Config {
    /// The loglevel to operate on (0 = info, 1 = debug, * = trace)
    #[arg(long = "loglevel", short = 'v', default_value_t = 0)]
    pub loglevel: u8,

    #[arg(long, default_value = DEFAULT_PARENT_DIR)]
    /// The parent directory to put the new formula into
    pub parent_dir: PathBuf,

    #[arg(long)]
    /// Preset specific architecture
    pub pkg_arch: Option<Vec<String>>,

    #[arg(long)]
    /// Preset the name
    pub pkg_name: Option<String>,

    #[arg(long)]
    /// Preset the version
    pub pkg_version: Option<String>,

    #[arg(long)]
    /// Preset the pkgver
    pub pkg_pkgver: Option<String>,

    #[arg(long)]
    /// Preset the description
    pub pkg_description: Option<String>,
}
