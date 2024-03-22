use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use log::debug;
use sys_mount::{UnmountDrop, UnmountFlags};

use crate::{
    error::{Error, ErrorExt},
    util,
};

use super::Mount;

/// Represents an overlayfs mount
pub struct OverlayMount {
    lower_dirs: Vec<PathBuf>,
    work_dir: PathBuf,
    upper_dir: PathBuf,
    merged_dir: PathBuf,

    mount: UnmountDrop<sys_mount::Mount>,
}

impl OverlayMount {
    /// Mounts an `overlayfs` using the supplied options
    /// # Arguments
    /// * `lower` -  The `lowerdir` sequence of lower directories
    /// * `work` - The `workdir`
    /// * `upper` - The `upperdir` to store the new files in
    /// * `merged` - The merged directory where to mount the `overlay` filesystem
    pub fn new(
        lower: Vec<PathBuf>,
        work: PathBuf,
        upper: PathBuf,
        merged: PathBuf,
    ) -> Result<OverlayMount, Error> {
        for d in &lower {
            util::fs::create_dir_all(d)?;
        }
        util::fs::create_dir_all(&work)?;
        util::fs::create_dir_all(&upper)?;
        util::fs::create_dir_all(&merged)?;

        let mut done: HashMap<PathBuf, ()> = HashMap::new();
        let mut lower_s = String::new();
        for p in lower.iter().rev() {
            if !done.contains_key(p) {
                done.insert(p.to_path_buf(), ());
                lower_s.push_str(&p.to_string_lossy());
                lower_s.push(':');
            } else {
                debug!("Deduplicated '{}'", p.to_string_lossy())
            }
        }
        // Pop the last colon
        lower_s.pop();

        let work_s = work.to_string_lossy();
        let upper_s = upper.to_string_lossy();

        let data = format!("lowerdir={lower_s},upperdir={upper_s},workdir={work_s}");
        debug!(
            "Mounting overlay ({}) ==> {}",
            &data,
            &merged.to_string_lossy()
        );

        let mount = sys_mount::Mount::builder()
            .fstype("overlay")
            .data(&data)
            .mount_autodrop("overlay", &merged, UnmountFlags::DETACH)
            .e_context(|| {
                format!(
                    "Mounting overlay ({}) => {}",
                    data,
                    merged.to_string_lossy()
                )
            })?;

        Ok(OverlayMount {
            lower_dirs: lower,
            work_dir: work,
            upper_dir: upper,
            merged_dir: merged,
            mount,
        })
    }
}

impl Mount for OverlayMount {
    fn get_fs_type(&self) -> String {
        "overlayfs".to_string()
    }

    fn get_target_path(&self) -> &Path {
        &self.merged_dir
    }

    fn get_source_path(&self) -> &Path {
        self.lower_dirs
            .first()
            .expect("Expexted at least 1 lowerdir")
    }

    fn get_source_paths(&self) -> Vec<&Path> {
        let mut vec: Vec<&Path> = self.lower_dirs.iter().map(|d| d.as_path()).collect();
        vec.push(&self.work_dir);
        vec.push(&self.upper_dir);

        vec
    }
}

impl Drop for OverlayMount {
    fn drop(&mut self) {
        debug!(
            "Unmounting {} at {}",
            self.get_fs_type(),
            self.mount.target_path().to_string_lossy()
        );
    }
}
