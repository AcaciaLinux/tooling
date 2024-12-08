use std::{io, path::PathBuf};

use clap::Parser;
use tooling::{
    error::{Error, ErrorExt, ErrorType},
    model::{odb_driver::FilesystemDriver, Object, ObjectDB, ObjectID, ObjectType},
    util::fs::{file_create, PathUtil},
};

use super::{common::Compression, Cli};

#[derive(Parser)]
pub struct CommandOdb {
    /// The command to execute
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    /// Retrieve an object from the object database
    Get {
        /// Put the object contents into a file instead of stdout
        #[arg(long, short)]
        output: Option<PathBuf>,

        /// The object id to retrieve
        oid: String,
    },
    /// Put a new object into the object database
    Put {
        /// The compression method to use
        #[arg(long, short, default_value_t = Compression::None)]
        compression: Compression,

        /// The path to the file to put into the object database
        path: PathBuf,
    },
    /// Pull an object from another object database
    Pull {
        /// The path to the other object database root
        #[arg(long)]
        other: PathBuf,

        /// The compression method to use
        #[arg(long, short, default_value_t = Compression::None)]
        compression: Compression,

        /// Whether to recursively pull dependencies or not
        #[arg(long, short, action)]
        recursive: bool,

        /// The object ID of the object to pull
        object: ObjectID,
    },
    /// Print the dependencies of an object
    Dependencies {
        /// List the dependencies in a tree form
        #[arg(long, action)]
        tree: bool,

        /// The object ID to list the dependencies of
        oid: ObjectID,
    },
}

impl CommandOdb {
    pub fn run(&self, cli: &Cli) -> Result<i32, Error> {
        let driver = FilesystemDriver::new(cli.get_home()?.object_db_path())?;
        let db = ObjectDB::init(Box::new(driver)).ctx(|| "Opening object db")?;

        self.command.run(cli, db)
    }
}

impl Command {
    pub fn run(&self, _cli: &Cli, mut odb: ObjectDB) -> Result<i32, Error> {
        match &self {
            Command::Get { output, oid } => {
                let oid = match ObjectID::new_from_hex(oid) {
                    Err(e) => {
                        return Err(Error::new_context(
                            ErrorType::Other(format!("Failed to parse object id: {}", e)),
                            "Parsing object id".to_string(),
                        ));
                    }
                    Ok(oid) => oid,
                };

                let mut object = odb.read(&oid)?;

                if let Some(output) = output {
                    let mut output_file =
                        file_create(output).e_context(|| "Creating output file")?;
                    io::copy(&mut object, &mut output_file)
                } else {
                    io::copy(&mut object, &mut io::stdout())
                }
                .e_context(|| "Copying object data")?;
            }
            Command::Put { compression, path } => {
                let object = odb
                    .insert_file(
                        path,
                        ObjectType::Other,
                        compression.clone().into(),
                        Vec::new(),
                    )
                    .e_context(|| format!("Putting {} into object database", path.str_lossy()))?;
                println!("{}", object.oid);
            }
            Command::Pull {
                other,
                compression,
                recursive,
                object,
            } => {
                let other_driver = FilesystemDriver::new(other.clone())?;
                let other_odb = ObjectDB::init(Box::new(other_driver))?;

                odb.pull(
                    &other_odb,
                    object.clone(),
                    compression.clone().into(),
                    *recursive,
                )?;
            }
            Command::Dependencies { tree, oid } => {
                let object = odb.get_object(oid)?;
                if *tree {
                    print_tree(&object, &odb, 0)?;
                } else {
                    let deps = object.resolve_dependencies(&odb, true)?;
                    for dep in deps {
                        println!("{}", dep.oid);
                    }
                }
            }
        }

        Ok(0)
    }
}

fn print_tree(object: &Object, odb: &ObjectDB, depth: u32) -> Result<(), Error> {
    if depth > 0 {
        println!("{}|--- {}", "|  ".repeat(depth as usize - 1), object.oid);
    } else {
        println!("{}", object.oid);
    }

    for dependency in object.resolve_dependencies(odb, false)? {
        print_tree(&dependency, odb, depth + 1)?;
    }

    Ok(())
}
