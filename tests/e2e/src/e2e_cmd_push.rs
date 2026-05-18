use anyhow::Result;

use crate::support::{cli, docker, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_push_invokes_remote_sync_path() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    repo::use_unreachable_ssh_port(&sandbox.path)?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["push"])?;
    cli::assert_failure(&output)?;
    cli::assert_stdout_contains(&output, "Syncing .bones/ to /tmp/e2eapp.git/bones/")?;

    Ok(())
}

#[test]
#[ignore = "requires docker"]
fn e2e_push_syncs_bones_directory_and_symlinks_remote_hooks() -> Result<()> {
    if !docker::docker_available() {
        return Ok(());
    }

    let _docker = docker::docker_session()?;

    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    docker::docker_exec("git init --bare /tmp/e2eapp.git")?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["push"])?;
    cli::assert_success(&output)?;
    cli::assert_stdout_contains(&output, ".bones/ synced to remote")?;

    let remote_config = docker::docker_exec_output("cat /tmp/e2eapp.git/bones/bones.yaml")?;
    assert!(remote_config.contains("project_name: e2eapp"));

    let pre_receive_target = docker::docker_exec_output("readlink /tmp/e2eapp.git/hooks/pre-receive")?;
    assert_eq!(pre_receive_target.trim(), "/tmp/e2eapp.git/bones/hooks/pre-receive");

    Ok(())
}
