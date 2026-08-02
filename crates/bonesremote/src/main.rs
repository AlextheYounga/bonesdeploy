mod cli;
mod commands;
mod privileges;
mod release;
mod ui;

use std::process::ExitCode;

use clap::Parser;
use commands::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match commands::run(&cli) {
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
    eprintln!("{} {head}", ui::failure_marker());
    for cause in chain {
        eprintln!("  caused by: {cause}");
    }
}
