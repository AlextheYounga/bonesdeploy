use anyhow::Result;
use std::process::Command;

use crate::support::{cli, docker, repo};

/// Verifies that `bonesdeploy pull` syncs the remote `.bones` folder back into the local project.
#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_pull_syncs_remote_bones_folder_back_locally() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    std::fs::write(
        sandbox.path.join(".bones/bones.yaml"),
        "data:\n  remote_name: production\n  project_name: e2eapp\n  port: \"2222\"\n  repo_path: /home/git/e2eapp.git\n  project_root: /srv/deployments/e2eapp\n  web_root: public\n  branch: master\n  deploy_on_push: true\npermissions:\n  defaults:\n    deploy_user: root\n    service_user: e2eapp\n    group: www-data\n    dir_mode: \"750\"\n    file_mode: \"640\"\nreleases:\n  keep: 5\n  shared_files:\n    - .env\n  shared_dirs:\n    - storage\nssl:\n  enabled: false\n  domain: \"\"\n  email: \"\"\n",
    )?;

    let _docker = docker::docker_session()?;

    docker::docker_exec("mkdir -p /home/git && git init --bare /home/git/e2eapp.git")?;
    docker::docker_exec("chown -R git:git /home/git/e2eapp.git")?;

    let status = Command::new("git")
        .args(["remote", "set-url", "production", "root@127.0.0.1:/home/git/e2eapp.git"])
        .current_dir(&sandbox.path)
        .status()?;
    assert!(status.success());

    let push_output = cli::run_bonesdeploy(&sandbox.path, ["push"])?;
    cli::assert_success(&push_output)?;

    std::fs::remove_dir_all(sandbox.path.join(".bones"))?;

    let pull_output = cli::run_bonesdeploy(&sandbox.path, ["pull"])?;
    cli::assert_success(&pull_output)?;

    let local_config = std::fs::read_to_string(sandbox.path.join(".bones/bones.yaml"))?;
    assert!(local_config.contains("project_name: e2eapp"));
    repo::assert_pre_push_symlink_exists(&sandbox.path)?;

    Ok(())
}
