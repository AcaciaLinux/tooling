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

        let _self = Self { root };

        fs::create_dir_all(&_self.get_tmp_dir()).e_context(|| "Creating tmp dir")?;

        Ok(_self)
    }

    /// Returns the root of the home directory
    pub fn get_root(&self) -> &Path {
        &self.root
    }

    /// Returns the path to the object database
    pub fn object_db_path(&self) -> PathBuf {
        self.root.join("objects")
    }

    /// Returns the path to a temporary directory
    /// in the home
    pub fn get_tmp_dir(&self) -> PathBuf {
        self.root.join("tmp")
    }

    /// Creates a file path for a temporary file
    /// that is unique within the temporary directory
    pub fn get_temp_file_path(&self) -> PathBuf {
        let uuid = uuid::Uuid::new_v4();
        self.get_tmp_dir().join(uuid.to_string())
    }

    /// Returns the path to the temporary build folders
    /// that are used to build the packages
    pub fn get_builds_dir(&self) -> PathBuf {
        self.get_tmp_dir().join("builds")
    }
}
