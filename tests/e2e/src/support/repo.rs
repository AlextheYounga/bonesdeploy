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
    let lib_dir = bones_dir.join(".lib");
    let hooks_dir = bones_dir.join("hooks");
    let deployment_dir = bones_dir.join("deployment");
    let playbook_dir = lib_dir.join("remote/playbooks");
    let roles_dir = lib_dir.join("remote/roles");

    fs::create_dir_all(&hooks_dir)?;
    fs::create_dir_all(&deployment_dir)?;
    fs::create_dir_all(&playbook_dir)?;
    fs::create_dir_all(&roles_dir)?;

    fs::write(lib_dir.join("hooks.sh"), "#!/usr/bin/env bash\n")?;
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

    let config = "data:\n  remote_name: production\n  project_name: e2eapp\n  port: \"2222\"\n  repo_path: /home/git/e2eapp.git\n  project_root: /srv/deployments/e2eapp\n  web_root: public\n  branch: master\n  deploy_on_push: true\npermissions:\n  defaults:\n    deploy_user: git\n    service_user: e2eapp\n    group: www-data\n    dir_mode: \"750\"\n    file_mode: \"640\"\nreleases:\n  keep: 5\n  shared_files:\n    - .env\n  shared_dirs:\n    - storage\nssl:\n  enabled: false\n  domain: \"\"\n  email: \"\"\n";
    fs::write(bones_dir.join("bones.yaml"), config)?;

    run_git(repo_root, ["remote", "add", "production", "git@127.0.0.1:/home/git/e2eapp.git"])?;

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

pub fn assert_bones_yaml_contains(repo_root: &Path, needle: &str) -> Result<()> {
    let content = fs::read_to_string(repo_root.join(".bones/bones.yaml"))?;
    if content.contains(needle) {
        return Ok(());
    }

    bail!("Expected .bones/bones.yaml to contain '{needle}', got:\n{content}")
}

pub fn install_real_site_assets(repo_root: &Path, workspace_root: &Path) -> Result<()> {
    let source = workspace_root.join("kit/.lib/remote");
    let target = repo_root.join(".bones/.lib/remote");

    if !source.is_dir() {
        bail!("Missing source remote assets directory: {}", source.display());
    }

    if target.exists() {
        fs::remove_dir_all(&target).with_context(|| format!("Failed to remove existing {}", target.display()))?;
    }

    copy_dir_recursive(&source, &target)
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("Failed to create {}", target.display()))?;

    for entry in fs::read_dir(source).with_context(|| format!("Failed to read {}", source.display()))? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)
                .with_context(|| format!("Failed to copy {} to {}", source_path.display(), target_path.display()))?;
        }
    }

    Ok(())
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
