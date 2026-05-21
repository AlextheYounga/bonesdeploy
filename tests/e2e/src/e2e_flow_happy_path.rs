use std::fs;
use std::os::unix::fs::PermissionsExt;

use anyhow::Result;

use crate::support::{cli, docker, repo};

#[test]
#[ignore = "requires docker"]
fn e2e_remote_happy_path_runs_push_deploy_and_rollback_in_sequence() -> Result<()> {
    let _docker = docker::docker_session()?;

    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    let pre_receive = sandbox.path.join(".bones/hooks/pre-receive");
    let post_receive = sandbox.path.join(".bones/hooks/post-receive");

    fs::write(
        &pre_receive,
        "#!/usr/bin/env bash\necho \"PRE BONES_FORCE_DEPLOY=${BONES_FORCE_DEPLOY} GIT_DIR=${GIT_DIR}\" >>/tmp/bonesdeploy-hooks.log\nexit 0\n",
    )?;
    fs::set_permissions(&pre_receive, fs::Permissions::from_mode(0o755))?;
    fs::write(
        &post_receive,
        "#!/usr/bin/env bash\necho \"POST BONES_FORCE_DEPLOY=${BONES_FORCE_DEPLOY} GIT_DIR=${GIT_DIR}\" >>/tmp/bonesdeploy-hooks.log\nexit 0\n",
    )?;
    fs::set_permissions(&post_receive, fs::Permissions::from_mode(0o755))?;

    let init = cli::run_bonesdeploy(&sandbox.path, ["init"])?;
    cli::assert_success(&init)?;

    let doctor = cli::run_bonesdeploy(&sandbox.path, ["doctor", "--local"])?;
    cli::assert_success(&doctor)?;

    docker::docker_exec("git init --bare /tmp/e2eapp.git")?;
    docker::docker_exec(
        "cat >/usr/local/bin/bonesremote <<'EOF'\n#!/usr/bin/env bash\necho \"$@\" >/tmp/bonesremote-invocation.log\nexit 0\nEOF\nchmod +x /usr/local/bin/bonesremote",
    )?;

    let push = cli::run_bonesdeploy(&sandbox.path, ["push"])?;
    cli::assert_success(&push)?;
    cli::assert_stdout_contains(&push, ".bones/ synced to remote")?;

    let deploy = cli::run_bonesdeploy(&sandbox.path, ["deploy"])?;
    cli::assert_success(&deploy)?;
    cli::assert_stdout_contains(&deploy, "Deployment complete")?;

    let rollback = cli::run_bonesdeploy(&sandbox.path, ["rollback"])?;
    cli::assert_success(&rollback)?;
    cli::assert_stdout_contains(&rollback, "Rollback complete")?;

    let hooks_log = docker::docker_exec_output("cat /tmp/bonesdeploy-hooks.log")?;
    assert!(hooks_log.contains("PRE BONES_FORCE_DEPLOY=1 GIT_DIR=/tmp/e2eapp.git"));
    assert!(hooks_log.contains("POST BONES_FORCE_DEPLOY=1 GIT_DIR=/tmp/e2eapp.git"));

    let rollback_invocation = docker::docker_exec_output("cat /tmp/bonesremote-invocation.log")?;
    assert!(rollback_invocation.contains("release rollback --config /tmp/e2eapp.git/bones/bones.yaml"));

    Ok(())
}
