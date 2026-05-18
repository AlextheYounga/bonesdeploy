use anyhow::Result;

use crate::support::{cli, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_version_prints_semver_banner() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    let output = cli::run_bonesdeploy(&sandbox.path, ["version"])?;
    cli::assert_success(&output)?;
    cli::assert_stdout_contains(&output, "bonesdeploy ")?;
    Ok(())
}
