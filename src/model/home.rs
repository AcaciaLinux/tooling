use std::path::{Path, PathBuf};

use log::debug;

use crate::{
    error::{Error, ErrorExt},
    util::fs::{self, PathUtil},
};

/// The home directory all tooling works in
pub struct Home {
    root: PathBuf,
}

impl Home {
    /// Opens or creates a new home directory
    pub fn new(root: PathBuf) -> Result<Self, Error> {
        debug!("Opening home @ {}", root.str_lossy());
        fs::create_dir_all(&root).e_context(|| format!("Creating home @ {}", root.str_lossy()))?;

        Ok(Self { root })
    }

    /// Returns the root of the home directory
    pub fn get_root(&self) -> &Path {
        &self.root
    }

    /// Returns the path to the object database
    pub fn object_db_path(&self) -> PathBuf {
        self.root.join("objects")
    }
}
