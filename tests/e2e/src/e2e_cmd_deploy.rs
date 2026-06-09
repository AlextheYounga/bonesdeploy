use anyhow::Result;

use crate::support::{cli, docker, repo};

/// Verifies that `bonesdeploy deploy` triggers the remote hook path end-to-end.
#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_deploy_invokes_remote_hook_path() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    let _docker = docker::docker_session()?;

    docker::docker_exec("mkdir -p /home/git && git init --bare /home/git/e2eapp.git")?;
    docker::docker_exec(
        "cat >/home/git/e2eapp.git/hooks/pre-receive <<'EOF'\n#!/usr/bin/env bash\necho \"PRE BONES_FORCE_DEPLOY=${BONES_FORCE_DEPLOY} GIT_DIR=${GIT_DIR}\" >>/tmp/bonesdeploy-hooks.log\nexit 0\nEOF\nchmod +x /home/git/e2eapp.git/hooks/pre-receive",
    )?;
    docker::docker_exec(
        "cat >/home/git/e2eapp.git/hooks/post-receive <<'EOF'\n#!/usr/bin/env bash\necho \"POST BONES_FORCE_DEPLOY=${BONES_FORCE_DEPLOY} GIT_DIR=${GIT_DIR}\" >>/tmp/bonesdeploy-hooks.log\nexit 0\nEOF\nchmod +x /home/git/e2eapp.git/hooks/post-receive",
    )?;
    let output = cli::run_bonesdeploy(&sandbox.path, ["deploy"])?;
    cli::assert_success(&output)?;

    let logs = docker::docker_exec_output("cat /tmp/bonesdeploy-hooks.log")?;
    assert!(logs.contains("PRE BONES_FORCE_DEPLOY=1 GIT_DIR=/home/git/e2eapp.git"));
    assert!(logs.contains("POST BONES_FORCE_DEPLOY=1 GIT_DIR=/home/git/e2eapp.git"));

    Ok(())
}
