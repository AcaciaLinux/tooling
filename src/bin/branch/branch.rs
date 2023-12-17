use std::{collections::HashMap, process::exit, sync::Arc};

use clap::Parser;
use log::{debug, error, info, warn};
use tooling::{
    abs_dist_dir, dist_dir,
    env::CustomExecutable,
    error::{Error, ErrorExt},
    package::{BuildIDProvider, CorePackage, InstallablePackage},
    tools::builder::{Builder, BuilderTemplate},
    util::{fs, signal::SignalDispatcher},
};

mod config;
use config::BuilderConfig;

fn run(signal_dispatcher: &SignalDispatcher, cli: BuilderConfig) -> Result<(), Error> {
    let context = || "Running the builder".to_string();

    let arch = cli.get_arch().clone();

    // Create a template for the builder
    let builder_template = BuilderTemplate {
        toolchain: cli.toolchain,
        workdir: cli.workdir,
        dist_dir: cli.dist_dir,
        overlay_dirs: cli.overlay_dirs,
        arch,
        formula_path: cli.formula,
    };

    // Create the builder
    let builder = Builder::from_template(builder_template).e_context(context)?;

    // If the `--exec` flag is passed, run a custom executable, else build normally
    if let Some(command) = cli.exec {
        let executable = CustomExecutable::new(command, HashMap::new());
        let env = builder.create_env().e_context(context)?;
        builder
            .run_custom(&env, &executable, signal_dispatcher)
            .e_context(context)?;
    } else {
        let pkg = builder.build(signal_dispatcher).e_context(context)?;

        if !cli.skip_validation {
            for file in &pkg.1 {
                for action in &file.actions {
                    let mut cmd = action.to_command(&file.path, &pkg.0, &abs_dist_dir())?;
                    debug!("Running {:?}", cmd);

                    let output = cmd.output().e_context(|| "Running sond".to_owned())?;
                    if !output.status.success() {
                        error!(
                            "Validation command {:?} failed: {}",
                            cmd,
                            String::from_utf8(output.stderr).unwrap()
                        )
                    }
                }
            }
        } else {
            warn!(
                "Skipping validation actions! This could leave this package in an unusable state!"
            );
        }

        for file in &pkg.1 {
            for error in &file.errors {
                warn!(
                    "VALIDATION for '{}' failed: {}",
                    file.path.to_string_lossy(),
                    error.oneline()
                )
            }
        }

        let installable = InstallablePackage::from_built(pkg.0, &dist_dir())?;
        fs::create_dir_all(&cli.output_dir)?;

        let archive_path = cli.output_dir.join(installable.get_archive_name());
        if archive_path.exists() {
            warn!(
                "Removing old build artifact at '{}'",
                archive_path.to_string_lossy()
            );
            fs::remove_file(&archive_path)
                .e_context(|| "Removing old package archive file".to_owned())?;
        }

        info!(
            "Archiving package to '{}'...",
            archive_path.to_string_lossy()
        );
        installable.archive(&archive_path)?;

        eprintln!(
            "Built package '{}' [{}] available at {}",
            installable.get_full_name(),
            installable.get_build_id(),
            archive_path.to_string_lossy()
        );
    }

    Ok(())
}

fn main() {
    // Parse command line arguments
    let cli = BuilderConfig::parse();

    if std::env::var("RUST_LOG").is_err() {
        match &cli.loglevel {
            0 => std::env::set_var("RUST_LOG", "info"),
            1 => std::env::set_var("RUST_LOG", "debug"),
            _ => std::env::set_var("RUST_LOG", "trace"),
        }
    }
    pretty_env_logger::init();

    let dispatcher = Arc::new(SignalDispatcher::default());
    let dsp_clone = dispatcher.clone();
    ctrlc::set_handler(move || dsp_clone.handle()).unwrap();

    match run(&dispatcher, cli) {
        Ok(()) => exit(0),
        Err(e) => {
            eprintln!("Failed to run builder: {e}");
            exit(1)
        }
    };
}
