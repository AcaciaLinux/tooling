extern crate colored;
use std::process::exit;

use colored::Colorize;

use clap::Parser;
use tooling::error::Error;

mod cli;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(v) => exit(v),
        Err(e) => {
            println!("{}", e.to_string().red())
        }
    }
}

async fn run() -> Result<i32, Error> {
    let cli = cli::Cli::parse();

    cli.run().await
}
