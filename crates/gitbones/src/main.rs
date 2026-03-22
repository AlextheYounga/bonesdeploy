mod commands;
mod config;
mod embedded;
mod git;
mod prompts;
mod ssh;

use anyhow::Result;
use clap::Parser;
use commands::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(&cli).await
}
