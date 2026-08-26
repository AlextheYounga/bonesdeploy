use crate::{config, infra};
use anyhow::Result;
use bonesdeploy_core::paths;
use std::path::Path;

pub fn run(format: &str) -> Result<()> {
    super::readiness::ensure_project_ready()?;

    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    bonesinfra::run_with_request(&["manifest", "show", "--request-stdin", "--format", format], &request)
}
