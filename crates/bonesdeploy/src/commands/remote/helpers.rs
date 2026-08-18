use anyhow::Result;
use console::style;

use bonesdeploy_core::paths;

use crate::ui::output;
use crate::ui::prompts;

pub fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_remote_helpers()? {
        println!("Skipped.");
        return Ok(());
    }

    println!("{}", style("Installing remote helper tools").cyan().bold());

    bonesinfra::run(&["helpers", "apply", "--env-file", paths::DOT_ENV])?;

    println!("{} Helper tools installed.", output::success_marker());
    Ok(())
}
