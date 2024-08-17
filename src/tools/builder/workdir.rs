use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::{
    error::{Error, ErrorExt},
    model::Home,
    util,
};

use lazy_static::lazy_static;

lazy_static! {
    /// The name of the install directory
    static ref PATH_INSTALL_DIR: PathBuf = PathBuf::from("install");
}

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

    /// The directory for the overlayfs to live in
    ///
    /// `<root>/overlay`
    pub fn get_overlay_dir(&self) -> PathBuf {
        self.root.join("overlay")
    }

    /// The directory for the overlayfs `work` dir to live in
    ///
    /// `<overlay_dir>/work`
    pub fn get_overlay_dir_work(&self) -> PathBuf {
        self.get_overlay_dir().join("work")
    }

    /// The directory for the overlayfs `upper` dir to live in
    ///
    /// `<overlay_dir>/upper`
    pub fn get_overlay_dir_upper(&self) -> PathBuf {
        self.get_overlay_dir().join("upper")
    }

    /// The directory for the overlayfs `merged` dir to live in
    ///
    /// `<overlay_dir>/merged`
    pub fn get_overlay_dir_merged(&self) -> PathBuf {
        self.get_overlay_dir().join("merged")
    }

    /// The directory for the formula and its data to live in
    ///
    /// `<root>/formula`
    pub fn get_formula_dir(&self) -> PathBuf {
        self.root.join("formula")
    }

    /// The path to the installation target directory from inside the `chroot`
    ///
    /// `/<PATH_INSTALL_DIR>`
    pub fn get_install_dir_inner(&self) -> PathBuf {
        PathBuf::from("/").join(&*PATH_INSTALL_DIR)
    }

    /// The path to the installation target directory from outside the `chroot`
    ///
    /// `<overlay_dir_merged>/<PATH_INSTALL_DIR>`
    pub fn get_install_dir_outer(&self) -> PathBuf {
        self.get_overlay_dir_merged().join(&*PATH_INSTALL_DIR)
    }

    /// The directory to place the finished artifact's output files in
    ///
    /// `<root>/out`
    pub fn get_output_dir(&self) -> PathBuf {
        self.get_root().join("out")
    }
}
