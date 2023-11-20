//! Bundles utility functions for mounting filesystems

use std::path::Path;

mod overlay;
pub use overlay::*;

use log::debug;
use sys_mount::{Mount, MountFlags, UnmountDrop, UnmountFlags};

/// Creates a bind mount from the source to the target using the `--bind` flag
/// # Arguments
/// * `source` - The source directory
/// * `target` - The target directory
/// * `readonly` - If the `RDONLY` flag should be appended
///
/// Mount command: `mount --bind <source> <target>`
pub fn mount_bind(
    source: &Path,
    target: &Path,
    readonly: bool,
) -> Result<UnmountDrop<Mount>, std::io::Error> {
    std::fs::create_dir_all(source)?;
    std::fs::create_dir_all(target)?;

    debug!(
        "Mounting bind {} ==> {}",
        source.to_string_lossy(),
        target.to_string_lossy()
    );

    if readonly {
        Mount::builder().flags(MountFlags::BIND | MountFlags::RDONLY)
    } else {
        Mount::builder().flags(MountFlags::BIND)
    }
    .mount_autodrop(source, target, UnmountFlags::DETACH)
}

/// Mounts a virtual kernel filesystem
/// # Arguments
/// * `filesystem` - The name of the filesystem (e.g. `proc`, `sysfs`)
/// * `target` - The path where to mount the filesystem
///
/// Mount command: `mount -t <filesystem> <filesystem> <target>`
pub fn mount_vkfs(filesystem: &str, target: &Path) -> Result<UnmountDrop<Mount>, std::io::Error> {
    std::fs::create_dir_all(target)?;

    debug!(
        "Mounting vkfs '{filesystem}' ==> {}",
        target.to_string_lossy()
    );

    let source_path = Path::new(filesystem);

    Mount::builder()
        .fstype(filesystem)
        .mount_autodrop(source_path, target, UnmountFlags::DETACH)
}
