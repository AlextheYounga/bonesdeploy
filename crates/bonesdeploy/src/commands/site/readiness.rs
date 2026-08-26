use std::path::Path;

use anyhow::{Result, bail};
use bonesdeploy_core::config::PROJECT_SETUP_ERROR;
use bonesdeploy_core::paths;

use crate::infra::git;

pub(super) fn ensure_project_ready() -> Result<()> {
    git::ensure_git_repository()?;
    let env_file = Path::new(paths::DOT_ENV);
    if !env_file.exists() || !Path::new(paths::LOCAL_INFRA_DIR).is_dir() {
        bail!(PROJECT_SETUP_ERROR);
    }
    Ok(())
}
