use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use tooling::{
    env::CustomExecutable,
    error::{Error, ErrorExt},
    files::package_index::PackageIndexFile,
    tools::builder::{Builder, BuilderTemplate},
    util::{parse::parse_toml, signal::SignalDispatcher},
};

mod config;
use config::BuilderConfig;

fn run(signal_dispatcher: &SignalDispatcher) -> Result<(), Error> {
    let context = || "Running the builder".to_string();

    // Parse command line arguments
    let cli = BuilderConfig::parse();

    let arch = cli.get_arch().clone();
    let package_index: PackageIndexFile = parse_toml(&cli.package_index)?;

    // Create a template for the builder
    let builder_template = BuilderTemplate {
        toolchain: cli.toolchain,
        workdir: cli.workdir,
        dist_dir: cli.acacia_dir,
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
        builder.build(signal_dispatcher).e_context(context)?;
    }

    Ok(())
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug")
    }
    pretty_env_logger::init();

    let dispatcher = Arc::new(SignalDispatcher::default());
    let dsp_clone = dispatcher.clone();
    ctrlc::set_handler(move || dsp_clone.handle()).unwrap();

    match run(&dispatcher) {
        Ok(()) => {}
        Err(e) => println!("Failed to run builder: {e}"),
    };
}
