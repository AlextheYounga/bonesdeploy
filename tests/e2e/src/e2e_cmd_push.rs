use anyhow::Result;

use crate::support::{cli, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_push_invokes_remote_sync_path() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["push"])?;
    cli::assert_failure(&output)?;

    Ok(())
}
