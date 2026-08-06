use anyhow::Result;

use super::{Context, config_repo_url};
use crate::infra::git::{output_at, run_at};

pub(super) const ID: &str = "0002-root-config-repo";

pub(super) fn apply(context: &Context<'_>) -> Result<()> {
    let remote_url = config_repo_url(context.cfg);
    if output_at(context.bones_dir, ["remote", "get-url", "origin"])?.status.success() {
        run_at(context.bones_dir, ["remote", "set-url", "origin", &remote_url])
    } else {
        run_at(context.bones_dir, ["remote", "add", "origin", &remote_url])
    }
}
