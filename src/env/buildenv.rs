use std::{
    io::{self},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use log::{debug, error, info, warn};
use std::process::Command;

use crate::{
    error::{Error, ErrorExt},
    util::{
        mount::{BindMount, Mount, VKFSMount},
        signal::SignalDispatcher,
    },
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
pub struct BuildEnvironment {
    /// The overlayfs that is used for the root
    root: Box<dyn Mount>,
    /// All the mounts that go into the build root
    mounts: Vec<Box<dyn Mount>>,
    /// The path to search for the host toolchain to prepend the PATH variable
    toolchain_dir: PathBuf,
}

impl BuildEnvironment {
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
        root_mount: Box<dyn Mount>,
        toolchain_dir: PathBuf,
    ) -> Result<BuildEnvironment, Error> {
        let context = || "Creating build environment";
        let target = root_mount.get_target_path();

        // Mount the virtual kernel filesystems
        let m_dev =
            BindMount::new(Path::new("/dev"), &target.join("dev"), false).e_context(context)?;
        let m_dev_pts = BindMount::new(
            Path::new("/dev/pts"),
            &target.join("dev").join("pts"),
            false,
        )?;
        let m_proc = VKFSMount::new("proc", &target.join("proc"))?;
        let m_sysfs = VKFSMount::new("sysfs", &target.join("sys"))?;
        let m_tmpfs = VKFSMount::new("tmpfs", &target.join("run"))?;

        Ok(BuildEnvironment {
            root: root_mount,
            mounts: vec![
                Box::new(m_dev),
                Box::new(m_dev_pts),
                Box::new(m_proc),
                Box::new(m_sysfs),
                Box::new(m_tmpfs),
            ],
            toolchain_dir,
        })
    }

    /// Adds a mount to the internal list of mounts to manage and eventually drop
    /// # Arguments
    /// * `mount` - The mount to add
    pub fn add_mount(&mut self, mount: Box<dyn Mount>) {
        self.mounts.push(mount);
    }

    /// Returns a reference to the `OverlayMount` used for the build environment
    pub fn get_root_mount(&self) -> &dyn Mount {
        self.root.as_ref()
    }
}

impl Environment for BuildEnvironment {
    fn execute(
        &self,
        executable: &dyn EnvironmentExecutable,
        signal_dispatcher: &SignalDispatcher,
    ) -> Result<std::process::ExitStatus, Error> {
        let mut command = Command::new("/bin/chroot");

        command
            .env_clear()
            .arg(self.root.get_target_path())
            .arg("env")
            .arg("-C")
            .arg(executable.get_workdir())
            .arg("sh")
            .arg("-e")
            .arg("-c")
            .arg(executable.get_command());

        let tc_dir = self.toolchain_dir.to_string_lossy();
        let path = format!(
            "/bin:/sbin:/usr/bin:/usr/sbin:{}/bin:{}/sbin",
            tc_dir, tc_dir
        );

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

        let executable_name = executable.get_name();
        let mut child = command
            .stdout(Stdio::piped())
            .spawn()
            .e_context(|| "Spawing child process".to_owned())?;

        // Get the `stdout` of the child to redirect it
        let mut child_stdout = child.stdout.take().expect("Stdout");

        let process_arc = Arc::new(Mutex::new(child));

        let handler_arc = process_arc.clone();

        thread::scope(|s| {
            // Construct a signal handler that will kill the child process
            let guard = signal_dispatcher.add_handler(Box::new(move || {
                match handler_arc.lock().expect("Lock handler mutex").kill() {
                    Ok(_) => warn!("Killed build step '{}'", executable_name),
                    Err(_) => error!("Failed to kill build step {}", executable_name),
                }
            }));

            // Redirect `stdout` of the child to `stderr`
            let _redirect_thread = s.spawn(|| {
                let mut stderr = io::stderr().lock();

                io::copy(&mut child_stdout, &mut stderr).expect("Redirect stderr");
            });

            // Loop until the child exits
            loop {
                // Lock the mutex to query
                let mut child = process_arc.lock().expect("Lock mutex");

                // If the child has exited, exit here, too
                if let Some(res) = child
                    .try_wait()
                    .e_context(|| "Waiting for child to join".to_owned())?
                {
                    debug!("Command exited with {}", res);
                    // Release the signal handler
                    drop(guard);

                    return Ok(res);
                }

                // Drop the mutex to free for the signal handler
                drop(child);
                std::thread::sleep(Duration::from_millis(100));
            }
        })
    }
}

impl Drop for BuildEnvironment {
    fn drop(&mut self) {
        info!("Tearing down build environment...");
        while let Some(mount) = self.mounts.pop() {
            drop(mount)
        }
    }
}
