mod cli;
mod commands;
mod privileges;
mod release;

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
    eprintln!("\x1b[1;31m✗\x1b[0m \x1b[1;31m{head}\x1b[0m");
    for cause in chain {
        eprintln!("  \x1b[2mcaused by: \x1b[0m\x1b[2m{cause}\x1b[0m");
    }
}
