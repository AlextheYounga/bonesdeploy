use anyhow::Result;

use crate::support::{cli, docker, repo};

// On a fresh VPS, deploy_user (git) does not exist yet — remote setup creates it via ansible.
// remote setup must connect as the bootstrap SSH user (root), not the deploy user.
// If it mistakenly uses deploy_user for SSH, the connection is refused because the user
// doesn't exist on a fresh server. This test guards against that regression.
#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_remote_setup_uses_bootstrap_user_not_deploy_user() -> Result<()> {
    let _docker = docker::docker_session()?;
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    repo::install_real_site_assets(&sandbox.path, &crate::support::paths::workspace_root())?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["remote", "setup"])?;
    cli::assert_failure(&output)?;
    cli::assert_stdout_contains(&output, "Running remote setup against 127.0.0.1 as root")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("as git"),
        "remote setup must not SSH as deploy user 'git'; the git user does not exist on a fresh VPS.\nstdout:\n{stdout}"
    );

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
