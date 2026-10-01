use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;
use console::style;

use crate::build;
use crate::config;
use crate::ui::output;

pub fn run() -> Result<()> {
    let config = config::load(Path::new(paths::DOT_ENV)).context("Failed to load root .env")?;

    println!("Building the committed {} branch locally...", config.branch);
    let artifact = build::package(&config)?;

    println!(
        "{} Built {} at {} ({} bytes).",
        output::success_marker(),
        style(&config.project_name).bold(),
        artifact.manifest.revision,
        artifact.manifest.artifact_length,
    );
    Ok(())
}
