use std::{io, path::PathBuf};

use clap::Parser;
use tooling::{
    error::{Error, ErrorExt, ErrorType},
    model::{ObjectDB, ObjectID, ObjectType},
    util::fs::{file_create, PathUtil},
    ODB_DEPTH,
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

        /// Insert new objects even if they already exist
        #[arg(long, short, default_value = "false")]
        force: bool,

        /// The path to the file to put into the object database
        path: PathBuf,
    },
}

impl CommandOdb {
    pub fn run(&self, cli: &Cli) -> Result<i32, Error> {
        let db = ObjectDB::init(cli.get_home()?.object_db_path(), ODB_DEPTH)?;

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
            Command::Put {
                compression,
                force,
                path,
            } => {
                let object = odb
                    .insert_file(
                        path,
                        ObjectType::Other,
                        compression.clone().into(),
                        !force,
                        Vec::new(),
                    )
                    .e_context(|| format!("Putting {} into object database", path.str_lossy()))?;
                println!("{}", object.oid);
            }
        }

        Ok(0)
    }
}
