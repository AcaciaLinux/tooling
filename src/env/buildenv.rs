use std::path::{Path, PathBuf};

use log::{debug, error, info};
use std::process::Command;
use sys_mount::{Mount, Unmount, UnmountDrop, UnmountFlags};

use crate::util::{
    mount::{mount_bind, mount_vkfs, OverlayMount},
    signal::SignalDispatcher,
};

use super::{Environment, EnvironmentExecutable};

/// Represents a build environment that can be used to build a package.
///
/// Expects the following directories in the `toolchain_dir`:
/// - `/bin`: Binaries
/// - `/sbin`: Superuser binaries
/// Expects the following programs:
/// - `env`: The `env` program that can be found using the PATH variable
/// - `sh`: The `sh` program that can be found using the PATH variable
pub struct BuildEnvironment<'a> {
    /// The overlayfs that is used for the root
    overlay: OverlayMount<'a>,
    /// All the mounts that go into the build root
    mounts: Vec<UnmountDrop<Mount>>,
    /// The path to search for the host toolchain to prepend the PATH variable
    toolchain_dir: PathBuf,
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
    /// * `toolchain_dir` - The directory to search for toolchain files (PATH)
    pub fn new(
        overlay_mount: OverlayMount<'a>,
        toolchain_dir: PathBuf,
    ) -> Result<BuildEnvironment<'a>, std::io::Error> {
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
            toolchain_dir,
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

impl<'a> Environment for BuildEnvironment<'a> {
    fn execute(
        &self,
        executable: &dyn EnvironmentExecutable,
        signal_dispatcher: &SignalDispatcher,
    ) -> Result<std::process::ExitStatus, std::io::Error> {
        let mut command = Command::new("/bin/chroot");

        command
            .env_clear()
            .arg(self.overlay.get_merged_dir())
            .arg("env")
            .arg("-C")
            .arg(executable.get_workdir())
            .arg("sh")
            .arg("-c")
            .arg(executable.get_command());

        let tc_dir = self.toolchain_dir.to_string_lossy();
        let path = format!("{}/bin:{}/sbin", tc_dir, tc_dir);

        command
            .env("PATH", path)
            .envs(executable.get_env_variables());

        debug!(
            "Running build step '{}', executing command 'chroot' with following arguments:",
            executable.get_name()
        );
        for arg in command.get_args() {
            debug!(" - {}", arg.to_string_lossy());
        }

        debug!("Following environment variables:");
        for env in command.get_envs() {
            if let Some(value) = env.1 {
                debug!(
                    " - {} = '{}'",
                    env.0.to_string_lossy(),
                    value.to_string_lossy()
                )
            } else {
                debug!(" - {}", env.0.to_string_lossy(),)
            }
        }

        let output = command.spawn()?.wait_with_output()?;

        debug!("Command exited with {}", output.status);

        Ok(output.status)
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
