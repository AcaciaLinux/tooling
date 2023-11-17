//! Bundles utility functions for mounting filesystems

use std::path::Path;

use log::debug;
use sys_mount::{Mount, Unmount, UnmountDrop, UnmountFlags};

/// Mounts an `overlayfs` using the supplied options
/// # Arguments
/// * `lower` -  The `lowerdir` sequence of lower directories
/// * `work` - The `workdir`
/// * `upper` - The `upperdir` to store the new files in
/// * `merged` - The merged directory where to mount the `overlay` filesystem
pub fn mount_overlay(
    lower: &[&Path],
    work: &Path,
    upper: &Path,
    merged: &Path,
) -> Result<UnmountDrop<Mount>, std::io::Error> {
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
        "Mounting overlay ({}) -> {}",
        &data,
        &merged.to_string_lossy()
    );

    let mount = Mount::builder()
        .fstype("overlay")
        .data(&data)
        .mount("overlay", merged)?;

    Ok(mount.into_unmount_drop(UnmountFlags::DETACH))
}
