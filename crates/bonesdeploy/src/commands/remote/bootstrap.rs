use anyhow::Result;

use bonesdeploy_core::paths;

use crate::ui::output;
use crate::ui::prompts;

pub fn run(skip_confirm: bool, show_next: bool) -> Result<()> {
    if !skip_confirm && !prompts::confirm_remote_setup()? {
        println!("Skipped.");
        return Ok(());
    }
    println!("Bootstrapping remote server...");

    bonesinfra::run(&[
        "setup",
        "apply",
        "--env-file",
        paths::DOT_ENV,
        "--bonesremote-version",
        env!("CARGO_PKG_VERSION"),
    ])?;

    println!("Remote bootstrap complete.");
    if show_next {
        println!();
        println!("{}", output::next_step("bonesdeploy remote runtime"));
    }

    Ok(())
}
