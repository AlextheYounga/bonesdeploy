use anyhow::Result;

use crate::support::{cli, docker, repo};

/// Verifies that `bonesdeploy push` invokes the remote sync path end-to-end.
#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_push_invokes_remote_sync_path() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    let _docker = docker::docker_session()?;

    docker::docker_exec("mkdir -p /home/git && git init --bare /home/git/e2eapp.git")?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["push"])?;
    cli::assert_success(&output)?;

    let remote_config = docker::docker_exec_output("cat /home/git/e2eapp.git/bones/bones.yaml")?;
    assert!(remote_config.contains("project_name: e2eapp"));

    let pre_receive_target = docker::docker_exec_output("readlink /home/git/e2eapp.git/hooks/pre-receive")?;
    assert_eq!(pre_receive_target.trim(), "/home/git/e2eapp.git/bones/hooks/pre-receive");

    Ok(())
}
