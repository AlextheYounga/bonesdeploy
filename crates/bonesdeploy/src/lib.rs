pub mod cli;
pub mod commands;
pub mod config;
pub mod frameworks;
pub mod infra;

mod ui;

use std::process::ExitCode;

use clap::Parser;
use console::style;

pub async fn run_cli() -> ExitCode {
    let cli = commands::Cli::parse();
    match commands::run(&cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_error(&error);
            ExitCode::FAILURE
        }
    }
}

fn print_error(error: &anyhow::Error) {
    let mut chain = error.chain();
    let Some(head) = chain.next() else {
        return;
    };
    eprintln!("{} {}", ui::output::failure_marker(), style(head).red().bold());
    for cause in chain {
        eprintln!("  {} {}", style("caused by:").dim(), style(cause).dim());
    }
}
