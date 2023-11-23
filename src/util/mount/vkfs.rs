use std::path::Path;

use log::debug;
use sys_mount::{UnmountDrop, UnmountFlags};

use super::Mount;

/// Represents a mounted kernel virtual filesystem
pub struct VKFSMount {
    source: String,

    mount: UnmountDrop<sys_mount::Mount>,
}

impl VKFSMount {
    /// Mounts a virtual kernel filesystem
    /// # Arguments
    /// * `filesystem` - The name of the filesystem (e.g. `proc`, `sysfs`)
    /// * `target` - The path where to mount the filesystem
    ///
    /// Mount command: `mount -t <filesystem> <filesystem> <target>`
    pub fn new(filesystem: &str, target: &Path) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(target)?;

        debug!(
            "Mounting vkfs '{filesystem}' ==> {}",
            target.to_string_lossy()
        );

        let source_path = Path::new(filesystem);

        let mount = sys_mount::Mount::builder()
            .fstype(filesystem)
            .mount_autodrop(source_path, target, UnmountFlags::DETACH)?;

        Ok(VKFSMount {
            mount,
            source: filesystem.to_string(),
        })
    }
}

impl Mount for VKFSMount {
    fn get_fs_type(&self) -> String {
        format!("vkfs ({})", self.source)
    }

    fn get_target_path(&self) -> &Path {
        self.mount.target_path()
    }
    fn get_source_path(&self) -> &Path {
        Path::new(&self.source)
    }

    fn get_source_paths(&self) -> Vec<&Path> {
        vec![Path::new(&self.source)]
    }
}

impl Drop for VKFSMount {
    fn drop(&mut self) {
        debug!(
            "Unmounting {} at {}",
            self.get_fs_type(),
            self.get_target_path().to_string_lossy()
        );
    }
}
