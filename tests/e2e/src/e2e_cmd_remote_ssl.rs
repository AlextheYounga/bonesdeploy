use anyhow::Result;

use crate::support::{cli, docker, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_remote_ssl_reaches_real_ssl_ansible_flow() -> Result<()> {
    let _docker = docker::docker_session()?;
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    repo::install_real_site_assets(&sandbox.path, &crate::support::paths::workspace_root())?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["remote", "ssl", "--domain", "app.test", "--email", "ops@test"])?;
    cli::assert_failure(&output)?;
    cli::assert_stdout_contains(&output, "Running remote ssl against 127.0.0.1 for app.test")?;
    cli::assert_stdout_contains(&output, "Ensuring python3 is available on remote host")?;

    repo::assert_bones_yaml_contains(&sandbox.path, "domain: app.test")?;
    repo::assert_bones_yaml_contains(&sandbox.path, "email: ops@test")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ansible-playbook failed")
            || stderr.contains("System has not been booted with systemd")
            || stderr.contains("Failed to connect to bus"),
        "Expected a meaningful SSL setup failure, got stderr:\n{stderr}"
    );

    Ok(())
}
