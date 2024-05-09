use std::{collections::HashSet, fs::File, path::PathBuf};

extern crate colored;
use clap::Parser;
use colored::Colorize;
use tooling::{
    error::{Error, ErrorExt},
    files::index::IndexFile,
    model::ObjectID,
    tools::indexer::Indexer,
    util::{
        fs::{self, PathUtil},
        Packable,
    },
};

#[derive(Parser)]
struct CliStat {
    /// The index file to stat
    index_file: PathBuf,
}

#[derive(Parser)]
struct CliIndex {
    #[arg(long, default_value_t = false)]
    stat: bool,

    /// Where to put the index file
    #[arg(long = "index-file", short)]
    index_file: PathBuf,

    /// The path to index
    path: PathBuf,
}

#[derive(Parser)]
enum Command {
    /// Provide statistics about an index file
    Stat(CliStat),
    /// Index a filesystem tree
    Index(CliIndex),
}

/// Work with Acacia indexes
#[derive(Parser)]
#[command(author = "Max Kofler", name = "branch", version)]
struct Cli {
    /// The loglevel to operate on (0 = info, 1 = debug, * = trace)
    #[arg(long = "loglevel", short = 'v', default_value_t = 0, global = true)]
    pub loglevel: u8,

    /// The command to execute
    #[command(subcommand)]
    command: Command,
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e.to_string().red())
        }
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    if std::env::var("RUST_LOG").is_err() {
        match &cli.loglevel {
            0 => std::env::set_var("RUST_LOG", "info"),
            1 => std::env::set_var("RUST_LOG", "debug"),
            _ => std::env::set_var("RUST_LOG", "trace"),
        }
    }
    pretty_env_logger::init();

    match cli.command {
        Command::Stat(cli) => command_stat(cli)?,
        Command::Index(cli) => command_index(cli)?,
    }

    Ok(())
}

fn print_stat(file: IndexFile) {
    let mut dir_ups = 0usize;
    let mut dirs = 0usize;
    let mut objects: HashSet<ObjectID> = HashSet::new();
    let mut symlinks = 0usize;
    for command in &file.commands {
        match command {
            tooling::util::fs::IndexCommand::DirectoryUP => {
                dir_ups += 1;
            }
            tooling::util::fs::IndexCommand::Directory { info: _, name: _ } => {
                dirs += 1;
            }
            tooling::util::fs::IndexCommand::File {
                info: _,
                name: _,
                oid,
            } => {
                objects.insert(oid.clone());
            }
            tooling::util::fs::IndexCommand::Symlink {
                info: _,
                name: _,
                dest: _,
            } => {
                symlinks += 1;
            }
        }
    }

    let duplicates: usize = file.commands.len() - (dir_ups + dirs + symlinks + objects.len());

    println!("Version:      {:>10}", file.version);
    println!();
    println!("UP:           {:>10}", dir_ups);
    println!("DIR:          {:>10}", dirs);
    println!("SYMLINKS:     {:>10}", symlinks);
    println!("OBJECTS:      {:>10}", objects.len());
    println!("--------------{:->10}", "");
    println!("Commands:     {:>10}", file.commands.len());
    println!("Duplicates:   {:>10}", duplicates);
}

fn command_stat(cli: CliStat) -> Result<(), Error> {
    let context = || format!("Providing stats about {}", &cli.index_file.str_lossy());

    let mut file_src = File::open(&cli.index_file).e_context(context)?;
    let file = IndexFile::unpack(&mut file_src)?;

    if let Some(file) = file {
        print_stat(file);
    }

    Ok(())
}

fn command_index(cli: CliIndex) -> Result<(), Error> {
    let context = || {
        format!(
            "Indexing '{}' to '{}'",
            cli.path.str_lossy(),
            cli.index_file.str_lossy()
        )
    };

    let mut file = fs::file_create(&cli.index_file).e_context(context)?;

    let indexer = Indexer::new(cli.path.clone());
    let index = indexer.run(true).e_context(context)?;

    let file_contents = index.to_index_file();
    file_contents.pack(&mut file).e_context(context)?;

    if cli.stat {
        print_stat(file_contents);
    }

    Ok(())
}
