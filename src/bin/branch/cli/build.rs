use std::sync::Arc;

use clap::Parser;
use log::info;
use tooling::{
    error::{Error, ErrorExt},
    model::{Formula, ObjectCompression, ObjectDB, ObjectID},
    tools::builder::Builder,
    util::signal::SignalDispatcher,
    ODB_DEPTH,
};

use super::Cli;

/// The `build` command
#[derive(Parser)]
pub struct BuildCommand {
    /// The compression to use for inserting the objects
    #[arg(long, short, default_value_t=ObjectCompression::Xz)]
    compression: ObjectCompression,

    /// The object id of the formula to be built
    formula: ObjectID,
}

impl BuildCommand {
    pub fn run(&self, cli: &Cli) -> Result<i32, Error> {
        let home = cli.get_home()?;

        info!("Building {}", self.formula);

        let object_db = ObjectDB::init(home.object_db_path(), ODB_DEPTH)?;

        let (formula, _object) =
            Formula::from_odb(&self.formula, &object_db).ctx(|| "Recalling formula object")?;

        let home = cli.get_home()?;
        let formula_name = formula.name.clone();

        let mut builder = Builder::new(&home, formula, &object_db).ctx(|| "Running the builder")?;

        let signal_dispatcher = Arc::new(SignalDispatcher::default());

        let sd_clone = signal_dispatcher.clone();
        ctrlc::set_handler(move || {
            sd_clone.handle();
        })
        .expect("Attach signal handler");

        builder
            .run(&signal_dispatcher)
            .ctx(|| format!("Building formula '{formula_name}'"))?;

        Ok(0)
    }
}
