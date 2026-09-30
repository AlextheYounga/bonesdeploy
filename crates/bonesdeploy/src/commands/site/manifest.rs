use crate::{config, infra};
use anyhow::Result;
use bonesdeploy_core::paths;
use std::path::Path;

pub fn run(format: &str) -> Result<()> {
    super::readiness::ensure_project_ready()?;

    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    let manifest =
        bonesinfra::run_with_request_output(&["manifest", "show", "--request-stdin", "--format", format], &request)?;
    print!("{manifest}");
    Ok(())
}
