use std::path::PathBuf;

use clap::Parser;
use log::info;
use tooling::{
    error::Error,
    files::formulafile::FormulaFile,
    model::ObjectCompression,
    util::{architecture::Architecture, fs::PathUtil},
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

        let (formula, object) =
            FormulaFile::parse_and_resolve(&self.file, &home, self.get_arch()?, self.compression)?;

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
