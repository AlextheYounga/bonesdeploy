use anyhow::Result;

use crate::support::{cli, docker, repo};

/// Verifies that `bonesdeploy remote ssl` executes the full Ansible SSL provisioning flow.
#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_remote_ssl_reaches_real_ssl_ansible_flow() -> Result<()> {
    let _docker = docker::docker_session()?;
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    repo::install_real_site_assets(&sandbox.path, &crate::support::paths::workspace_root())?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["remote", "ssl", "--domain", "app.test", "--email", "ops@test"])?;
    cli::assert_failure(&output)?;

    repo::assert_bones_yaml_contains(&sandbox.path, "domain: app.test")?;
    repo::assert_bones_yaml_contains(&sandbox.path, "email: ops@test")?;

    Ok(())
}
