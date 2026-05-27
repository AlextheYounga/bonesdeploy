use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::support::{cli, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_init_reuses_existing_scaffold_and_symlinks_pre_push_hook() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["init"])?;
    cli::assert_success(&output)?;
    repo::assert_pre_push_symlink_exists(&sandbox.path)?;

    Ok(())
}

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_init_does_not_claim_to_create_the_remote_user() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    std::fs::write(
        sandbox.path.join(".bones/bones.yaml"),
        "data:\n  remote_name: production\n  project_name: e2eapp\n  host: 127.0.0.1\n  port: \"22\"\n  repo_path: /home/git/e2eapp.git\n  project_root: /srv/deployments/e2eapp\n  web_root: public\n  branch: master\n  deploy_on_push: true\npermissions:\n  defaults:\n    deploy_user: git\n    service_user: e2eapp\n    group: www-data\n    dir_mode: \"750\"\n    file_mode: \"640\"\nreleases:\n  keep: 5\n  shared_files:\n    - .env\n  shared_dirs:\n    - storage\nssl:\n  enabled: false\n  domain: \"\"\n  email: \"\"\n",
    )?;

    let status = Command::new("git")
        .args(["remote", "remove", "production"])
        .current_dir(&sandbox.path)
        .status()
        .context("Failed to remove test remote")?;
    if !status.success() {
        bail!("Failed to remove test remote");
    }

    let output = cli::run_bonesdeploy(&sandbox.path, ["init"])?;
    cli::assert_success(&output)?;
    cli::assert_stdout_contains(
        &output,
        "Configured local git remote production -> git@127.0.0.1:/home/git/e2eapp.git",
    )?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Added git remote production ->"), "stdout was:\n{stdout}");

    Ok(())
}
