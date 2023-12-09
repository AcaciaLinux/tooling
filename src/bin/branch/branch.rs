use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use log::warn;
use tooling::{
    abs_dist_dir, dist_dir,
    env::CustomExecutable,
    error::{Error, ErrorExt},
    files::package_index::PackageIndexFile,
    package::{BuildIDProvider, CorePackage, InstallablePackage, PathPackage},
    tools::builder::{Builder, BuilderTemplate},
    util::{parse::parse_toml, signal::SignalDispatcher},
};

mod config;
use config::BuilderConfig;

fn run(signal_dispatcher: &SignalDispatcher, cli: BuilderConfig) -> Result<(), Error> {
    let context = || "Running the builder".to_string();

    let arch = cli.get_arch().clone();
    let package_index: PackageIndexFile = parse_toml(&cli.package_index)?;

    // Create a template for the builder
    let builder_template = BuilderTemplate {
        toolchain: cli.toolchain,
        workdir: cli.workdir,
        dist_dir: cli.dist_dir,
        overlay_dirs: cli.overlay_dirs,
        arch,
        formula_path: cli.formula,
        package_index_provider: &package_index,
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

        for file in pkg.1 {
            for error in file.errors {
                warn!(
                    "VALIDATION for '{}' failed: {error}",
                    file.path.to_string_lossy()
                )
            }
            for action in file.actions {
                let cmd = action.to_command(&file.path, &pkg.0, &abs_dist_dir())?;
                println!("{}", cmd);
            }
        }

        let installable = InstallablePackage::from_built(pkg.0, &dist_dir())?;
        eprintln!(
            "Built package '{}' [{}] available at {}",
            installable.get_full_name(),
            installable.get_build_id(),
            installable.get_real_path().to_string_lossy()
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
        Ok(()) => {}
        Err(e) => eprintln!("Failed to run builder: {e}"),
    };
}
