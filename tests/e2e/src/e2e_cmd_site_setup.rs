use anyhow::Result;

use crate::support::{cli, docker, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_site_setup_reaches_real_remote_ansible_flow() -> Result<()> {
    let _docker = docker::docker_session()?;
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    repo::install_real_site_assets(&sandbox.path, &crate::support::paths::workspace_root())?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["site", "setup"])?;
    cli::assert_failure(&output)?;
    cli::assert_stdout_contains(&output, "Running site setup against 127.0.0.1 as root")?;
    cli::assert_stdout_contains(&output, "Ensuring python3 is available on remote host")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ansible-playbook failed")
            || stderr.contains("System has not been booted with systemd")
            || stderr.contains("Failed to connect to bus"),
        "Expected a meaningful remote setup failure, got stderr:\n{stderr}"
    );

    Ok(())
}
