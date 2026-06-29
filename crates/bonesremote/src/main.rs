mod cli;
mod commands;
mod privileges;
mod release;
mod release_state;

use anyhow::Result;
use clap::Parser;
use commands::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(&cli)
}
