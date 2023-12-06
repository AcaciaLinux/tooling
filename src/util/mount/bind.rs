use std::path::{Path, PathBuf};

use log::debug;
use sys_mount::{MountFlags, UnmountDrop, UnmountFlags};

use crate::error::{Error, ErrorExt};

use super::Mount;

/// Represents a bind mount
pub struct BindMount {
    readonly: bool,
    source: PathBuf,

    mount: UnmountDrop<sys_mount::Mount>,
}

impl BindMount {
    /// Creates a bind mount from the source to the target using the `--bind` flag
    /// # Arguments
    /// * `source` - The source directory
    /// * `target` - The target directory
    /// * `readonly` - If the `RDONLY` flag should be appended
    ///
    /// Mount command: `mount --bind <source> <target>`
    pub fn new(source: &Path, target: &Path, readonly: bool) -> Result<Self, Error> {
        std::fs::create_dir_all(source).e_context(|| {
            format!(
                "Creating bind mount source directory {}",
                source.to_string_lossy()
            )
        })?;
        std::fs::create_dir_all(target).e_context(|| {
            format!(
                "Creating bind mount target directory {}",
                target.to_string_lossy()
            )
        })?;

        debug!(
            "Mounting bind {} ==> {}",
            source.to_string_lossy(),
            target.to_string_lossy()
        );

        let mount = if readonly {
            sys_mount::Mount::builder().flags(MountFlags::BIND | MountFlags::RDONLY)
        } else {
            sys_mount::Mount::builder().flags(MountFlags::BIND)
        }
        .mount_autodrop(source, target, UnmountFlags::DETACH)
        .e_context(|| {
            format!(
                "Bind mounting {} to {}",
                source.to_string_lossy(),
                target.to_string_lossy()
            )
        })?;

        Ok(Self {
            mount,
            readonly,
            source: source.to_path_buf(),
        })
    }
}

impl Mount for BindMount {
    fn get_fs_type(&self) -> String {
        format!("bind (readonly={})", self.readonly)
    }

    fn get_target_path(&self) -> &Path {
        self.mount.target_path()
    }

    fn get_source_path(&self) -> &Path {
        &self.source
    }

    fn get_source_paths(&self) -> Vec<&Path> {
        vec![&self.source]
    }
}

impl Drop for BindMount {
    fn drop(&mut self) {
        debug!(
            "Unmounting {} at {}",
            self.get_fs_type(),
            self.get_target_path().to_string_lossy()
        );
    }
}
