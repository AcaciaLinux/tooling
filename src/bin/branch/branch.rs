extern crate colored;
use std::process::exit;

use colored::Colorize;

use clap::Parser;
use tooling::error::Error;

mod cli;

fn main() {
    match run() {
        Ok(v) => exit(v),
        Err(e) => {
            println!("{}", e.to_string().red())
        }
    }
}

fn run() -> Result<i32, Error> {
    let cli = cli::Cli::parse();

    cli.run()
}
