use std::path::Path;

use log::{debug, error, info};
use sys_mount::{Mount, Unmount, UnmountDrop, UnmountFlags};

use crate::util::mount::{mount_bind, mount_vkfs, OverlayMount};

/// Represents a build environment that can be used to build a package
pub struct BuildEnvironment<'a> {
    /// The overlayfs that is used for the root
    overlay: OverlayMount<'a>,
    /// All the mounts that go into the build root
    mounts: Vec<UnmountDrop<Mount>>,
}

impl<'a> BuildEnvironment<'a> {
    /// Creates a new build environment from the `overlay_mount`, mounting in the following vkfs:
    /// - `/dev (bind)`==> `<merged>/dev`
    /// - `/dev/pts (bind)`==> `<merged>/dev/pts`
    /// - `proc (vkfs)`==> `<merged>/proc`
    /// - `sysfs (vkfs)`==> `<merged>/sys`
    /// - `tmpfs (vkfs)`==> `<merged>/run`
    /// # Arguments
    /// * `overlay_mount` - The overlay mount to construct the build environment in
    pub fn new(overlay_mount: OverlayMount<'a>) -> Result<BuildEnvironment<'a>, std::io::Error> {
        let target = overlay_mount.get_merged_dir();

        // Mount the virtual kernel filesystems
        let m_dev = mount_bind(Path::new("/dev"), &target.join("dev"), false)?;
        let m_dev_pts = mount_bind(
            Path::new("/dev/pts"),
            &target.join("dev").join("pts"),
            false,
        )?;
        let m_proc = mount_vkfs("proc", &target.join("proc"))?;
        let m_sysfs = mount_vkfs("sysfs", &target.join("sys"))?;
        let m_tmpfs = mount_vkfs("tmpfs", &target.join("run"))?;

        Ok(BuildEnvironment {
            overlay: overlay_mount,
            mounts: vec![m_dev, m_dev_pts, m_proc, m_sysfs, m_tmpfs],
        })
    }

    /// Adds a mount to the internal list of mounts to manage and eventually drop
    /// # Arguments
    /// * `mount` - The mount to add
    pub fn add_mount(&mut self, mount: UnmountDrop<Mount>) {
        self.mounts.push(mount);
    }

    /// Returns a reference to the `OverlayMount` used for the build environment
    pub fn get_overlay_mount(&self) -> &OverlayMount {
        &self.overlay
    }
}

impl<'a> Drop for BuildEnvironment<'a> {
    fn drop(&mut self) {
        info!("Tearing down build environment...");
        while let Some(mount) = self.mounts.pop() {
            match mount.unmount(UnmountFlags::DETACH) {
                Err(e) => error!(
                    "Failed to unmount {}: {}",
                    mount.target_path().to_string_lossy(),
                    e
                ),
                Ok(_) => {
                    debug!("Unmounted {}", mount.target_path().to_string_lossy())
                }
            }
        }
    }
}
