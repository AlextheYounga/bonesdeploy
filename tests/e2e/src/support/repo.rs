use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use tempfile::TempDir;

pub struct TempGitRepo {
    _temp_dir: TempDir,
    pub path: PathBuf,
}

pub fn create_temp_git_repo() -> Result<TempGitRepo> {
    let temp_dir = tempfile::tempdir().context("Failed to create temp dir")?;
    let path = temp_dir.path().to_path_buf();

    run_git(&path, ["init"])?;
    run_git(&path, ["config", "user.name", "bonesdeploy-e2e"])?;
    run_git(&path, ["config", "user.email", "bonesdeploy-e2e@test.local"])?;

    fs::write(path.join("README.md"), "# e2e fixture\n").context("Failed to write fixture README")?;
    run_git(&path, ["add", "README.md"])?;
    run_git(&path, ["commit", "-m", "initial"])?;

    Ok(TempGitRepo { _temp_dir: temp_dir, path })
}

pub fn write_minimal_bones_project(repo_root: &Path) -> Result<()> {
    let bones_dir = repo_root.join(".bones");
    let hooks_dir = bones_dir.join("hooks");
    let deployment_dir = bones_dir.join("deployment");
    let playbook_dir = bones_dir.join("site/playbooks");
    let roles_dir = bones_dir.join("site/roles");

    fs::create_dir_all(&hooks_dir)?;
    fs::create_dir_all(&deployment_dir)?;
    fs::create_dir_all(&playbook_dir)?;
    fs::create_dir_all(&roles_dir)?;

    fs::write(bones_dir.join("hooks.sh"), "#!/usr/bin/env bash\n")?;
    let pre_push = hooks_dir.join("pre-push");
    let pre_receive = hooks_dir.join("pre-receive");
    let post_receive = hooks_dir.join("post-receive");
    let deployment_script = deployment_dir.join("01_run_deployment_concerns.sh");
    fs::write(&pre_push, "#!/usr/bin/env bash\n")?;
    fs::write(&pre_receive, "#!/usr/bin/env bash\n")?;
    fs::write(&post_receive, "#!/usr/bin/env bash\n")?;
    fs::write(&deployment_script, "#!/usr/bin/env bash\n")?;
    fs::set_permissions(&pre_push, fs::Permissions::from_mode(0o755))?;
    fs::set_permissions(&pre_receive, fs::Permissions::from_mode(0o755))?;
    fs::set_permissions(&post_receive, fs::Permissions::from_mode(0o755))?;
    fs::set_permissions(&deployment_script, fs::Permissions::from_mode(0o755))?;
    fs::write(playbook_dir.join("setup.yml"), "---\n- hosts: all\n  tasks: []\n")?;

    let config = "data:\n  remote_name: production\n  project_name: e2eapp\n  host: 127.0.0.1\n  port: \"2222\"\n  git_dir: /tmp/e2eapp.git\n  branch: master\n  deploy_on_push: true\npermissions:\n  defaults:\n    deploy_user: root\n    service_user: e2eapp\n    group: www-data\n    dir_mode: \"750\"\n    file_mode: \"640\"\nreleases:\n  keep: 5\n  shared_paths:\n    - .env\n    - storage\nssl:\n  enabled: false\n  domain: \"\"\n  email: \"\"\n";
    fs::write(bones_dir.join("bones.yaml"), config)?;

    run_git(repo_root, ["remote", "add", "production", "root@127.0.0.1:/tmp/e2eapp.git"])?;

    Ok(())
}

pub fn assert_pre_push_symlink_exists(repo_root: &Path) -> Result<()> {
    let link = repo_root.join(".git/hooks/pre-push");
    let metadata = fs::symlink_metadata(&link).context("Missing pre-push hook link")?;
    if !metadata.file_type().is_symlink() {
        bail!("Expected {} to be symlink", link.display());
    }

    let target = fs::read_link(&link)?;
    let expected = Path::new("../../.bones/hooks/pre-push");
    if target != expected {
        bail!("Unexpected pre-push target: {}", target.display());
    }

    Ok(())
}

pub fn use_unreachable_ssh_port(repo_root: &Path) -> Result<()> {
    let bones_yaml = repo_root.join(".bones/bones.yaml");
    let content = fs::read_to_string(&bones_yaml)?;
    fs::write(&bones_yaml, content.replace("port: \"2222\"", "port: \"1\""))?;

    Ok(())
}

pub fn assert_bones_yaml_contains(repo_root: &Path, needle: &str) -> Result<()> {
    let content = fs::read_to_string(repo_root.join(".bones/bones.yaml"))?;
    if content.contains(needle) {
        return Ok(());
    }

    bail!("Expected .bones/bones.yaml to contain '{needle}', got:\n{content}")
}

fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) -> Result<()> {
    let output = Command::new("git").args(args).current_dir(cwd).output().context("Failed to run git command")?;

    if output.status.success() {
        return Ok(());
    }

    bail!(
        "git command failed in {} with args {:?}.\nstdout:\n{}\nstderr:\n{}",
        cwd.display(),
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
