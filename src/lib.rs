//! This crate provides tooling for the [AcaciaLinux](https://github.com/AcaciaLinux) project

#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;

use base64::engine::GeneralPurpose;
use base64::prelude::BASE64_URL_SAFE;

/// The architecture string to use for architecture-independent packages
pub static ANY_ARCH: &str = "any";

/// The DIST directory. Default is `acacia`. This **HAS** to be relative to be able to join onto other paths
pub static DIST_DIR: &str = "acacia";

/// The HOME directory. Default is `~/.acacia`
pub static HOME_DIR: &str = ".acacia";

/// The file type suffix for the package archive file
pub static PACKAGE_ARCHIVE_FILE_SUFFIX: &str = ".tar.xz";

/// The file type suffix for an object file
pub static OBJECT_FILE_EXTENSION: &str = "aobj";

/// The base64 engine
pub static BASE64_ENGINE: GeneralPurpose = BASE64_URL_SAFE;

/// Provide a relative `PathBuf` pointing to the `DIST_DIR`
pub fn dist_dir() -> PathBuf {
    PathBuf::from(DIST_DIR)
}

/// Provide an absolute `PathBuf` pointing to the `DIST_DIR` relative to `/`
pub fn abs_dist_dir() -> PathBuf {
    PathBuf::from("/").join(dist_dir())
}

/// The commit hash of the commit this binary was built from
const GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");

pub mod assert;
pub mod cache;
pub mod env;
pub mod error;
pub mod files;
pub mod model;
pub mod package;
pub mod tools;
pub mod util;
