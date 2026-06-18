mod app;
mod cli;
mod commands;
mod config;

mod infra;
pub(crate) use infra::{bootstrap_ssh, embedded, git, python, ssh};

mod ui;
pub(crate) use ui::prompts;

use anyhow::Result;
use clap::Parser;
use commands::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(&cli).await
}
