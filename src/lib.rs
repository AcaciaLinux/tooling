//! This crate provides tooling for the [AcaciaLinux](https://github.com/AcaciaLinux) project

/// The commit hash of the commit this binary was built from
const GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");

pub mod env;
pub mod error;
pub mod files;
pub mod package;
pub mod tools;
pub mod util;
pub mod validators;
