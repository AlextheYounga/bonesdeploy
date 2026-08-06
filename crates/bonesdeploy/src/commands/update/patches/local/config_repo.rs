use anyhow::{Result, bail};

use super::{Context, config_repo_url};
use crate::infra::git::{is_repository_at, output_at, run_at};

pub(super) const ID: &str = "0001-config-repo";

pub(super) fn apply(context: &Context<'_>) -> Result<()> {
    if !is_repository_at(context.bones_dir)? {
        run_at(context.bones_dir, ["init", "--initial-branch", "master"])?;
    }

    let remote_url = config_repo_url(context.cfg);
    let origin = output_at(context.bones_dir, ["remote", "get-url", "origin"])?;
    if origin.status.success() {
        let actual_url = String::from_utf8_lossy(&origin.stdout).trim().to_string();
        if actual_url != remote_url {
            bail!("origin points to {actual_url}, expected {remote_url}");
        }
        return Ok(());
    }

    run_at(context.bones_dir, ["remote", "add", "origin", &remote_url])
}
