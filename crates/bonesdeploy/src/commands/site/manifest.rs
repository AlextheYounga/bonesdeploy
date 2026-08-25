use anyhow::Result;
use bonesdeploy_core::paths;

pub fn run(format: &str) -> Result<()> {
    super::readiness::ensure_project_ready()?;

    bonesinfra::run(&["manifest", "show", "--env-file", paths::DOT_ENV, "--format", format])
}
