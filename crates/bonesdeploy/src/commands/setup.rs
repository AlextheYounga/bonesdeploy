use anyhow::{Context, Result};

use crate::commands::{server, site};
use crate::ui::output;

pub async fn run(skip_confirm: bool) -> Result<()> {
    server::setup(skip_confirm).await.with_context(|| setup_error("setting up the server baseline"))?;
    site::setup(skip_confirm).await.with_context(|| setup_error("setting up the site"))
}

fn setup_error(step: &str) -> String {
    format!(
        "Setup failed while {step}.\n\nNext: fix the error above, then {} again.",
        output::run_command("bonesdeploy setup")
    )
}
