use std::path::PathBuf;

use clap::Parser;
use tooling::{
    error::Error,
    model::{odb_driver::FilesystemDriver, Formula, ObjectCompression, ObjectDB, ObjectID},
    tools::builder::Builder,
    util::{architecture::Architecture, signal::SignalDispatcher},
};

use super::Cli;

/// The `ingest` command
#[derive(Parser)]
pub struct BuildCommand {
    /// The compression to use for inserting the objects
    #[arg(long, short, default_value_t=ObjectCompression::Xz)]
    compression: ObjectCompression,

    /// The architecture to ingest the formula for
    #[arg(long, short)]
    pub architecture: Option<Architecture>,

    #[arg(long)]
    pub lower: Vec<PathBuf>,

    formula: ObjectID,
}

impl BuildCommand {
    pub fn run(&self, cli: &Cli) -> Result<i32, Error> {
        let home = cli.get_home()?;

        let driver = FilesystemDriver::new(home.object_db_path())?;
        let odb = ObjectDB::init(Box::new(driver))?;

        let (formula, _object) = Formula::from_odb(&self.formula, &odb)?;

        println!("{:#?}", formula);

        let builder = Builder::new(&home, formula);

        let dispatcher = SignalDispatcher::default();

        builder.build(self.lower.clone(), &dispatcher)?;

        Ok(0)
    }
}
