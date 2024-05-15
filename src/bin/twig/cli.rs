use std::path::PathBuf;

use clap::Parser;
use tooling::{
    error::{Error, ErrorType},
    model::Home,
};

mod odb;

#[derive(Parser)]
pub struct Cli {
    /// The log level to operate on (0 = info, 1 = debug, * = trace)
    #[arg(long = "loglevel", short = 'v', default_value_t = 0, global = true)]
    pub loglevel: u8,

    /// The home directory where all Acacia tooling works in [~/.acacia]
    #[arg(long)]
    home: Option<PathBuf>,

    /// The command to execute
    #[command(subcommand)]
    command: TwigCommand,
}

#[derive(Parser)]
pub enum TwigCommand {
    /// Perform operations on or with the object database
    Odb(odb::CommandOdb),
}

impl Cli {
    pub fn run(&self) -> Result<i32, Error> {
        if std::env::var("RUST_LOG").is_err() {
            match &self.loglevel {
                0 => std::env::set_var("RUST_LOG", "info"),
                1 => std::env::set_var("RUST_LOG", "debug"),
                _ => std::env::set_var("RUST_LOG", "trace"),
            }
        }
        pretty_env_logger::init();

        self.command.run(self)?;

        Ok(0)
    }

    pub fn get_home(&self) -> Result<Home, Error> {
        let home = match &self.home {
            Some(root) => Home::new(root.clone()),
            None => match home::home_dir() {
                Some(home_dir) => Home::new(home_dir.join(tooling::HOME_DIR)),
                None => {
                    return Err(Error::new(ErrorType::Other(
                        "Home cannot be determined, use '--home'".to_owned(),
                    )))
                }
            },
        }?;

        Ok(home)
    }
}

impl TwigCommand {
    pub fn run(&self, cli: &Cli) -> Result<i32, Error> {
        match self {
            Self::Odb(cmd) => cmd.run(cli),
        }
    }
}
