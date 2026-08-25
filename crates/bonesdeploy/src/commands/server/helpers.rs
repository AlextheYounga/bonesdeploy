use anyhow::Result;
use bonesdeploy_core::paths;
use console::style;

use crate::ui::output;
use crate::ui::prompts;

pub fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_server_helpers()? {
        println!("Skipped.");
        return Ok(());
    }

    println!("{}", style("Installing server helper tools").cyan().bold());
    bonesinfra::run(&["helpers", "apply", "--env-file", paths::DOT_ENV])?;
    println!("{} Helper tools installed.", output::success_marker());
    Ok(())
}
