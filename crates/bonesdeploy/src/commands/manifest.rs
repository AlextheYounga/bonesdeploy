use std::path::Path;

use anyhow::{Result, bail};
use bonesdeploy_core::config::PROJECT_SETUP_ERROR;

use crate::infra::git;
use bonesdeploy_core::paths;

pub fn run(format: &str) -> Result<()> {
    git::ensure_git_repository()?;

    let env_file = Path::new(paths::DOT_ENV);
    if !env_file.exists() || !Path::new(paths::LOCAL_INFRA_DIR).is_dir() {
        bail!(PROJECT_SETUP_ERROR);
    }

    bonesinfra::run(&["manifest", "show", "--env-file", paths::DOT_ENV, "--format", format])
}
