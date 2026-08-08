use anyhow::{Context, Result};

use crate::config;
use crate::infra::ssh;

use super::{Patch, Version};

pub(super) async fn apply(
    session: &openssh::Session,
    cfg: &config::Bones,
    target: Version,
    patches: &[Patch],
) -> Result<()> {
    for patch in patches.iter().filter(|patch| patch.introduced_in <= target) {
        let command = format!(
            "bonesremote patch apply --site {site} --patch {patch}",
            site = ssh::shell_quote(&cfg.project_name),
            patch = ssh::shell_quote(patch.id),
        );
        ssh::run_cmd(session, &command).await.with_context(|| format!("Remote patch {} failed", patch.id))?;
    }
    Ok(())
}
