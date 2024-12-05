use std::path::PathBuf;

use clap::Parser;
use tooling::{
    error::{Error, ErrorExt},
    model::{odb_driver::FilesystemDriver, ObjectDB, ObjectID, Tree},
    util::{fs::PathUtil, ODBUnpackable},
};

use super::{common::Compression, Cli};

#[derive(Parser)]
pub struct CommandTree {
    /// The command to execute
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    /// Create a new tree by indexing a filesystem tree
    Create {
        /// The compression to apply to the indexed objects
        #[arg(long, short, default_value_t = Compression::Xz)]
        compression: Compression,

        /// Display a stat of the created tree
        #[arg(long, default_value_t = false)]
        stat: bool,

        /// The path to index
        path: PathBuf,
    },
    /// Deploy a tree to a directory
    Deploy {
        /// The object id of the tree to deploy
        #[arg(long, short)]
        tree: ObjectID,

        /// The directory to deploy to
        root: PathBuf,
    },
    /// List the contents of a tree file
    List {
        /// The object id of the tree to read
        oid: ObjectID,
    },
}

impl CommandTree {
    pub fn run(&self, cli: &Cli) -> Result<i32, Error> {
        self.command.run(cli)
    }
}

impl Command {
    fn run(&self, cli: &Cli) -> Result<i32, Error> {
        match self {
            Command::Create {
                compression,
                stat,
                path,
            } => {
                let context = || format!("Indexing {}", path.str_lossy(),);

                let driver = FilesystemDriver::new(cli.get_home()?.object_db_path())?;
                let mut db = ObjectDB::init(Box::new(driver)).ctx(|| "Opening object db")?;

                let tree = Tree::index(path, &mut db, compression.clone().into()).ctx(context)?;

                let tree_object = tree
                    .insert_into_odb(&mut db, compression.clone().into())
                    .ctx(|| "Inserting the tree")
                    .ctx(context)?;

                if *stat {
                    for cmd in &tree.entries {
                        println!("{cmd}");
                    }
                }

                println!("{}", tree_object.oid);
            }
            Command::Deploy { tree, root } => {
                let driver = FilesystemDriver::new(cli.get_home()?.object_db_path())?;
                let db = ObjectDB::init(Box::new(driver)).ctx(|| "Opening object db")?;

                let mut tree_object = db.read(tree).ctx(|| "Opening tree object")?;

                let tree =
                    Tree::unpack_from_odb(&mut tree_object, &db).ctx(|| "Reading tree object")?;
                tree.deploy(root, &db).ctx(|| "Deploying tree")?;
            }
            Command::List { oid } => {
                let driver = FilesystemDriver::new(cli.get_home()?.object_db_path())?;
                let db = ObjectDB::init(Box::new(driver)).ctx(|| "Opening object db")?;

                let mut object = db.read(oid).ctx(|| "Reading tree object")?;
                let tree =
                    Tree::unpack_from_odb(&mut object, &db).ctx(|| "Reading object contents")?;

                for cmd in tree.entries {
                    println!("{cmd}");
                }
            }
        }

        Ok(0)
    }
}
/*
fn print_stat(file: IndexFile) {
    let mut dir_ups = 0usize;
    let mut dirs = 0usize;
    let mut objects: HashSet<ObjectID> = HashSet::new();
    let mut symlinks = 0usize;
    /*for command in &file.commands {
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
    */

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
*/
