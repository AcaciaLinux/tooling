use std::io::{self, stderr, Write};

use clap::Parser;
use config::Config;
use tooling::{
    error::{Error, ErrorExt},
    files::formula::{FormulaFile, FormulaPackage, FormulaPackageSource},
    util::{fs::create_dir_all, parse::write_toml},
};

mod config;

/// Runs the builder
fn run(cli: Config) -> Result<(), Error> {
    let context = || "Generating formula files".to_owned();

    let arch = match &cli.pkg_arch {
        Some(arch) => arch.clone(),
        None => vec![prompt_stdin("Package architecture >>").e_context(context)?],
    };

    let name = match &cli.pkg_name {
        Some(preset) => preset.clone(),
        None => prompt_stdin("Package name >>").e_context(context)?,
    };
    let version = match &cli.pkg_version {
        Some(preset) => preset.clone(),
        None => prompt_stdin("Package version >>").e_context(context)?,
    };
    let description = match &cli.pkg_version {
        Some(preset) => preset.clone(),
        None => prompt_stdin("Package description >>").e_context(context)?,
    };
    let source = prompt_stdin("Main source URL >>").e_context(context)?;
    let source = source
        .replace(&name, "$PKG_NAME")
        .replace(&version, "$PKG_VERSION");

    let sources = if !source.is_empty() {
        Some(vec![FormulaPackageSource {
            url: source,
            dest: None,
            extract: true,
        }])
    } else {
        None
    };

    let package = FormulaPackage {
        name: name.clone(),
        version: version.clone(),
        description: description.clone(),
        host_dependencies: None,
        target_dependencies: None,
        extra_dependencies: None,
        strip: true,
        arch: Some(arch),
        prepare: Some("cd $PKG_NAME-$PKG_VERSION && ".to_owned()),
        build: Some("cd $PKG_NAME-$PKG_VERSION && ".to_owned()),
        check: Some("cd $PKG_NAME-$PKG_VERSION && ".to_owned()),
        package: Some("cd $PKG_NAME-$PKG_VERSION && ".to_owned()),
        sources,
    };

    let formula = FormulaFile {
        version: 1,
        package,
    };

    let dir_path = cli.parent_dir.join(&name);
    let file_path = dir_path.join(format!("{}.toml", name));
    create_dir_all(&dir_path).e_context(context)?;

    write_toml(&file_path, &formula).e_context(context)?;

    Ok(())
}

fn main() {
    // Parse command line arguments
    let cli = Config::parse();

    if std::env::var("RUST_LOG").is_err() {
        match &cli.loglevel {
            0 => std::env::set_var("RUST_LOG", "info"),
            1 => std::env::set_var("RUST_LOG", "debug"),
            _ => std::env::set_var("RUST_LOG", "trace"),
        }
    }
    pretty_env_logger::init();

    match run(cli) {
        Ok(()) => {}
        Err(e) => eprintln!("Failed to run formulagen: {e}"),
    };
}

/// Ouputs the `prompt` and reads from stdin and runs `trim()` on the incoming string
fn prompt_stdin(prompt: &str) -> Result<String, Error> {
    eprint!("{} ", prompt);
    stderr()
        .flush()
        .e_context(|| "Reading from stdin".to_owned())?;

    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin
        .read_line(&mut buffer)
        .e_context(|| "Reading from stdin".to_owned())?;

    Ok(buffer.trim().to_string())
}
