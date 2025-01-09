use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::{
    error::{Error, ErrorExt},
    model::Home,
    util,
};

/// A working directory for the builder to work in
#[derive(Debug)]
pub struct BuilderWorkdir {
    /// The root of the working directory
    root: PathBuf,
    /// The build's unique ID
    id: String,
}

impl BuilderWorkdir {
    /// Creates a new workdir at `<root>/<id>`
    /// # Arguments
    /// * `root` - The directory the workdir exists in
    pub fn new(home: &Home) -> Result<Self, Error> {
        let id = Uuid::new_v4().to_string();
        let root = home.get_builds_dir().join(&id);

        util::fs::create_dir_all(&root)
            .e_context(|| format!("Creating workdir root @ {}", root.to_string_lossy()))?;

        Ok(Self { root, id })
    }

    /// Returns the build id for this working directory
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Returns the root directory for this working directory
    pub fn get_root(&self) -> &Path {
        &self.root
    }
}
