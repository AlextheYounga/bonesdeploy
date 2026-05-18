use anyhow::Result;

use crate::support::{cli, docker, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_rollback_invokes_remote_release_rollback() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    repo::use_unreachable_ssh_port(&sandbox.path)?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["rollback"])?;
    cli::assert_failure(&output)?;
    cli::assert_stdout_contains(&output, "Rolling back e2eapp on 127.0.0.1")?;

    Ok(())
}

#[test]
#[ignore = "requires docker"]
fn e2e_rollback_calls_remote_bonesremote_release_rollback_with_expected_config_path() -> Result<()> {
    let _docker = docker::docker_session()?;

    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    docker::docker_exec(
        "cat >/usr/local/bin/bonesremote <<'EOF'\n#!/usr/bin/env bash\necho \"$@\" >/tmp/bonesremote-invocation.log\nexit 0\nEOF\nchmod +x /usr/local/bin/bonesremote",
    )?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["rollback"])?;
    cli::assert_success(&output)?;
    cli::assert_stdout_contains(&output, "Rollback complete")?;

    let invocation = docker::docker_exec_output("cat /tmp/bonesremote-invocation.log")?;
    assert!(invocation.contains("release rollback --config /tmp/e2eapp.git/bones/bones.yaml"));

    Ok(())
}
