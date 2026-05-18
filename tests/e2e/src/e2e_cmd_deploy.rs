use anyhow::Result;

use crate::support::{cli, docker, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_deploy_invokes_remote_hook_path() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    let _docker = docker::docker_session()?;

    docker::docker_exec("git init --bare /tmp/e2eapp.git")?;
    docker::docker_exec(
        "cat >/tmp/e2eapp.git/hooks/pre-receive <<'EOF'\n#!/usr/bin/env bash\necho \"PRE BONES_FORCE_DEPLOY=${BONES_FORCE_DEPLOY} GIT_DIR=${GIT_DIR}\" >>/tmp/bonesdeploy-hooks.log\nexit 0\nEOF\nchmod +x /tmp/e2eapp.git/hooks/pre-receive",
    )?;
    docker::docker_exec(
        "cat >/tmp/e2eapp.git/hooks/post-receive <<'EOF'\n#!/usr/bin/env bash\necho \"POST BONES_FORCE_DEPLOY=${BONES_FORCE_DEPLOY} GIT_DIR=${GIT_DIR}\" >>/tmp/bonesdeploy-hooks.log\nexit 0\nEOF\nchmod +x /tmp/e2eapp.git/hooks/post-receive",
    )?;
    let output = cli::run_bonesdeploy(&sandbox.path, ["deploy"])?;
    cli::assert_success(&output)?;
    cli::assert_stdout_contains(&output, "Deployment complete")?;

    let logs = docker::docker_exec_output("cat /tmp/bonesdeploy-hooks.log")?;
    assert!(logs.contains("PRE BONES_FORCE_DEPLOY=1 GIT_DIR=/tmp/e2eapp.git"));
    assert!(logs.contains("POST BONES_FORCE_DEPLOY=1 GIT_DIR=/tmp/e2eapp.git"));

    Ok(())
}
