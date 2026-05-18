use anyhow::Result;

use crate::support::{cli, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_site_setup_invokes_ansible_flow() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["site", "setup"])?;
    cli::assert_failure(&output)?;

    Ok(())
}
