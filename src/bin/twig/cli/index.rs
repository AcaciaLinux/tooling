use std::{collections::HashSet, path::PathBuf};

use clap::Parser;
use tooling::{
    error::{Error, ErrorExt},
    files::index::IndexFile,
    model::{ObjectDB, ObjectID},
    tools::indexer::Indexer,
    util::{
        fs::{self, PathUtil},
        Packable,
    },
};

use super::{common::Compression, Cli};

#[derive(Parser)]
pub struct CommandIndex {
    /// The command to execute
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    /// Create a new index by indexing a filesystem tree
    Create {
        /// The output file
        #[arg(long, short)]
        output: PathBuf,

        /// The compression to apply to the indexed objects
        #[arg(long, short, default_value_t = Compression::Xz)]
        compression: Compression,

        /// Display a stat of the created index file
        #[arg(long, default_value_t = false)]
        stat: bool,

        /// Force overwriting of existing objects
        #[arg(long, short, default_value_t = false)]
        force: bool,

        /// The path to index
        path: PathBuf,
    },
}

impl CommandIndex {
    pub fn run(&self, cli: &Cli) -> Result<i32, Error> {
        self.command.run(cli)
    }
}

impl Command {
    fn run(&self, cli: &Cli) -> Result<i32, Error> {
        match self {
            Command::Create {
                output,
                compression,
                stat,
                force,
                path,
            } => {
                let context = || {
                    format!(
                        "Indexing '{}' to '{}'",
                        path.str_lossy(),
                        output.str_lossy()
                    )
                };

                let mut file = fs::file_create(output).e_context(context)?;
                let mut db = ObjectDB::init(cli.get_home()?.object_db_path(), 5)
                    .e_context(|| "Opening object database")?;

                let indexer = Indexer::new(path.clone());
                let index = indexer
                    .run(true, &mut db, compression.clone().into(), !force)
                    .e_context(context)?;

                let file_contents = index.to_index_file();
                file_contents.pack(&mut file).e_context(context)?;

                if *stat {
                    print_stat(file_contents);
                }
            }
        }
        Ok(0)
    }
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
