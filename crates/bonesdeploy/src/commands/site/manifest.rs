use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::paths;
use bonesinfra::manifest::inspect;

use crate::config;
use crate::ui::manifest;

pub async fn run(format: &str) -> Result<()> {
    super::readiness::ensure_project_ready()?;

    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let report = inspect(&cfg, Path::new(".")).await?;
    if format == "json" {
        print!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }

    print!("{}", manifest::render_report(report)?);
    Ok(())
}
