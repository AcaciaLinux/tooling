use std::path::Path;

use log::debug;
use sys_mount::{Mount, UnmountDrop, UnmountFlags};

pub struct OverlayMount<'a> {
    lower_dirs: &'a [&'a Path],
    work_dir: &'a Path,
    upper_dir: &'a Path,
    merged_dir: &'a Path,

    mount: UnmountDrop<Mount>,
}

impl<'a> OverlayMount<'a> {
    /// Mounts an `overlayfs` using the supplied options
    /// # Arguments
    /// * `lower` -  The `lowerdir` sequence of lower directories
    /// * `work` - The `workdir`
    /// * `upper` - The `upperdir` to store the new files in
    /// * `merged` - The merged directory where to mount the `overlay` filesystem
    pub fn new(
        lower: &'a [&'a Path],
        work: &'a Path,
        upper: &'a Path,
        merged: &'a Path,
    ) -> Result<OverlayMount<'a>, std::io::Error> {
        std::fs::create_dir_all(work)?;
        std::fs::create_dir_all(upper)?;
        std::fs::create_dir_all(merged)?;

        let mut lower_s = String::new();
        for p in lower {
            lower_s.push_str(&p.to_string_lossy());
            lower_s.push(':');
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

        let mount = Mount::builder()
            .fstype("overlay")
            .data(&data)
            .mount_autodrop("overlay", merged, UnmountFlags::DETACH)?;

        Ok(OverlayMount {
            lower_dirs: lower,
            work_dir: work,
            upper_dir: upper,
            merged_dir: merged,
            mount,
        })
    }

    /// Returns the array of lowerdirs this overlay mount consists of
    pub fn get_lower_dirs(&self) -> &[&Path] {
        self.lower_dirs
    }

    /// Returns the workdir this overlay mount consists of
    pub fn get_work_dir(&self) -> &Path {
        self.work_dir
    }

    /// Returns the upperdir this overlay mount consists of
    pub fn get_upper_dir(&self) -> &Path {
        self.upper_dir
    }

    /// Returns the target directory this overlay mount points to
    pub fn get_merged_dir(&self) -> &Path {
        self.merged_dir
    }
}

impl<'a> Drop for OverlayMount<'a> {
    fn drop(&mut self) {
        debug!(
            "Unmounting overlayfs {}",
            self.mount.target_path().to_string_lossy()
        );
    }
}
