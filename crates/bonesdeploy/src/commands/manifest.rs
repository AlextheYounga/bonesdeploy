use std::path::Path;

use anyhow::{Result, bail};

use crate::infra::git;
use bonesdeploy_core::paths;

pub fn run(format: &str) -> Result<()> {
    git::ensure_git_repository()?;

    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if !bones_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::LOCAL_BONES_TOML);
    }

    bonesinfra::run(&["manifest", "show", "--config", paths::LOCAL_BONES_TOML, "--format", format])
}
