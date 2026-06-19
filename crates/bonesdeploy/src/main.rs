mod cli;
mod commands;
mod config;

mod infra;

mod ui;

use anyhow::Result;
use clap::Parser;
use commands::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(&cli).await
}
