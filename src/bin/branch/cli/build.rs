use std::{collections::HashMap, path::PathBuf, sync::Arc};

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

/// Parse a single key-value pair
fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

/// The `build` command
#[derive(Parser)]
pub struct BuildCommand {
    /// The compression to use for inserting the objects
    #[arg(long, short, default_value_t=ObjectCompression::Xz)]
    compression: ObjectCompression,

    /// Define an additional environment variable to pass to the builder
    ///
    /// This will `taint` the build
    #[arg(short = 'E', value_parser = parse_key_val::<String, String>)]
    environment_variables: Vec<(String, String)>,

    /// Add an additional lower directory to the lower directories of the builder
    ///
    /// This will `taint` the build
    #[arg(short = 'L')]
    additional_lowerdirs: Vec<PathBuf>,

    /// The object id of the formula to be built
    formula: ObjectID,
}

impl BuildCommand {
    pub fn run(&self, cli: &Cli) -> Result<i32, Error> {
        let home = cli.get_home()?;

        info!("Building {}", self.formula);

        let mut object_db = ObjectDB::init(home.object_db_path(), ODB_DEPTH)?;

        let (formula, _object) =
            Formula::from_odb(&self.formula, &object_db).ctx(|| "Recalling formula object")?;

        let home = cli.get_home()?;
        let formula_name = formula.name.clone();

        let mut builder =
            Builder::new(&home, formula, &mut object_db).ctx(|| "Running the builder")?;

        let signal_dispatcher = Arc::new(SignalDispatcher::default());

        let sd_clone = signal_dispatcher.clone();
        ctrlc::set_handler(move || {
            sd_clone.handle();
        })
        .expect("Attach signal handler");

        let mut envs = HashMap::new();

        for (key, val) in self.environment_variables.clone() {
            envs.insert(key, val);
        }

        builder.add_env_vars(envs);
        builder.add_lower_dirs(self.additional_lowerdirs.clone());

        let index_obj = builder
            .run(&signal_dispatcher, self.compression, true)
            .ctx(|| format!("Building formula '{formula_name}'"))?;

        println!("{}", index_obj.oid);

        Ok(0)
    }
}
