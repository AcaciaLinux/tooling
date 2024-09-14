use std::path::PathBuf;

use clap::Parser;
use log::info;
use tooling::{
    error::{Error, ErrorExt},
    files::formulafile::FormulaFile,
    model::ObjectCompression,
    util::{
        architecture::Architecture,
        fs::{file_read_to_string, PathUtil},
    },
};

use super::Cli;

/// The `ingest` command
#[derive(Parser)]
pub struct IngestCommand {
    /// The compression to use for inserting the objects
    #[arg(long, short, default_value_t=ObjectCompression::Xz)]
    compression: ObjectCompression,

    /// The architecture to ingest the formula for
    #[arg(long, short)]
    pub architecture: Option<Architecture>,

    /// The file to the formula to be ingested
    file: PathBuf,
}

impl IngestCommand {
    pub fn run(&self, cli: &Cli) -> Result<i32, Error> {
        let home = cli.get_home()?;

        let formula_string = file_read_to_string(&self.file)?;
        let formula_file: FormulaFile =
            toml::from_str(&formula_string).e_context(|| "Parsing formula source")?;

        let formula_parent = self
            .file
            .parent()
            .expect("Parent for formula file")
            .to_owned();

        let (formula, object) = formula_file
            .resolve(&home, formula_parent, self.get_arch()?, self.compression)
            .e_context(|| "Resolving formula")?;

        info!(
            "Ingested {} -> {}:\n{:#?}",
            self.file.str_lossy(),
            object.oid,
            formula
        );
        println!("{}", object.oid);
        Ok(0)
    }

    /// Returns the configured architecture, using the host
    /// architecture in case none is specified
    pub fn get_arch(&self) -> Result<Architecture, Error> {
        match &self.architecture {
            Some(arch) => Ok(arch.clone()),
            None => Architecture::new_uname(),
        }
    }
}
