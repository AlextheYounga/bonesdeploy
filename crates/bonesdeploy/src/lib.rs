pub mod build;
pub mod cli;
pub mod commands;
pub mod config;
pub mod frameworks;
pub mod infra;

mod platform;
mod ui;

#[cfg(test)]
pub(crate) mod test_support {
    use std::env;
    use std::ffi::OsString;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn with_env<F, T>(key: &str, value: Option<&str>, test: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _lock = lock_env();
        let previous = env::var_os(key);
        set(key, value);
        let result = test();
        restore(key, previous);
        result
    }

    fn set(key: &str, value: Option<&str>) {
        // Environment mutation is synchronized above because Rust 2024 marks
        // the process-wide environment API as unsafe.
        // SAFETY: callers hold ENV_LOCK for the complete temporary mutation.
        unsafe {
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }
    }

    fn restore(key: &str, value: Option<OsString>) {
        // See `set`: the lock prevents concurrent tests from observing this
        // temporary process-wide environment change.
        // SAFETY: callers hold ENV_LOCK for the complete temporary mutation.
        unsafe {
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }
    }
}

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
