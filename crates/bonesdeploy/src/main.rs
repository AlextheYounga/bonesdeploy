mod cli;
mod commands;
mod config;

mod infra;
mod runtimes;

mod ui;

use std::process::ExitCode;

use clap::Parser;
use commands::Cli;
use console::style;
use ui::output;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
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
    eprintln!("{} {}", output::failure_marker(), style(head).red().bold());
    for cause in chain {
        eprintln!("  {} {}", style("caused by:").dim(), style(cause).dim());
    }
}
