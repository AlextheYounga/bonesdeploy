use anyhow::Result;

use crate::support::{cli, docker, repo};

#[test]
#[ignore = "requires docker"]
fn e2e_remote_happy_path_runs_full_lifecycle_in_sequence() -> Result<()> {
    let _docker = docker::docker_session()?;

    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    repo::install_real_site_assets(&sandbox.path, &crate::support::paths::workspace_root())?;

    let init = cli::run_bonesdeploy(&sandbox.path, ["init"])?;
    cli::assert_success(&init)?;

    let remote_setup = cli::run_bonesdeploy(&sandbox.path, ["remote", "setup"])?;
    cli::assert_success(&remote_setup)?;
    cli::assert_stdout_contains(&remote_setup, "Remote setup complete")?;

    let push = cli::run_bonesdeploy(&sandbox.path, ["push"])?;
    cli::assert_success(&push)?;
    cli::assert_stdout_contains(&push, ".bones/ synced to remote")?;

    let deploy = cli::run_bonesdeploy(&sandbox.path, ["deploy"])?;
    cli::assert_success(&deploy)?;
    cli::assert_stdout_contains(&deploy, "Deployment complete")?;

    let rollback = cli::run_bonesdeploy(&sandbox.path, ["rollback"])?;
    cli::assert_success(&rollback)?;
    cli::assert_stdout_contains(&rollback, "Rollback complete")?;

    Ok(())
}
