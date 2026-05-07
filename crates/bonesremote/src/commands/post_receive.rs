use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::privileges;
use crate::release_state;

use super::wire_release;

pub fn run(config_path: &str, revision: Option<&str>) -> Result<()> {
    privileges::ensure_not_root("bonesremote hooks post-receive")?;

    let cfg = config::load(Path::new(config_path))?;
    let build_root = release_state::build_root(&cfg);

    if !build_root.exists() {
        bail!("Build workspace does not exist: {}", build_root.display());
    }

    let checkout_target = revision.unwrap_or(cfg.data.branch.as_str());
    println!("Checking out {checkout_target} to {}...", build_root.display());

    let status = Command::new("git")
        .arg("--work-tree")
        .arg(&build_root)
        .arg("--git-dir")
        .arg(&cfg.data.git_dir)
        .arg("checkout")
        .arg("-f")
        .arg(checkout_target)
        .status()
        .with_context(|| {
            format!("Failed to run git checkout for target '{checkout_target}' into {}", build_root.display())
        })?;

    if !status.success() {
        bail!("git checkout failed for target '{checkout_target}': status {status}");
    }

    wire_release::run(config_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::run;
    use crate::config::Constants;
    use crate::release_state;

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("bonesremote_post_receive_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    fn run_command(command: &mut Command, label: &str) -> Result<()> {
        let status = command.status()?;
        anyhow::ensure!(status.success(), "Command failed ({label}) with status {status}");
        Ok(())
    }

    fn write_config(path: &Path, git_dir: &Path, deploy_root: &Path, branch: &str) -> Result<()> {
        let yaml = format!(
            "data:\n  remote_name: production\n  project_name: postreceive\n  host: localhost\n  port: \"22\"\n  git_dir: {}\n  live_root: {}\n  deploy_root: {}\n  branch: {branch}\n  deploy_on_push: true\npermissions:\n  defaults:\n    deploy_user: git\n    service_user: postreceive\n    group: www-data\n    dir_mode: \"750\"\n    file_mode: \"640\"\nreleases:\n  keep: 5\n  shared_paths:\n    - .env\n",
            git_dir.display(),
            deploy_root.join("live").display(),
            deploy_root.display()
        );
        fs::write(path, yaml)?;
        Ok(())
    }

    fn create_remote_with_master_commit(root: &Path) -> Result<PathBuf> {
        let bare = root.join("repo.git");
        let work = root.join("work");

        run_command(Command::new("git").args(["init", "--bare", bare.to_string_lossy().as_ref()]), "git init --bare")?;
        run_command(Command::new("git").args(["init", work.to_string_lossy().as_ref()]), "git init work")?;
        run_command(
            Command::new("git").args(["-C", work.to_string_lossy().as_ref(), "config", "user.name", "Unit Test"]),
            "git config user.name",
        )?;
        run_command(
            Command::new("git").args([
                "-C",
                work.to_string_lossy().as_ref(),
                "config",
                "user.email",
                "unit@test.local",
            ]),
            "git config user.email",
        )?;

        fs::write(work.join("README.md"), "hello\n")?;
        run_command(Command::new("git").args(["-C", work.to_string_lossy().as_ref(), "add", "."]), "git add")?;
        run_command(
            Command::new("git").args(["-C", work.to_string_lossy().as_ref(), "commit", "-m", "initial"]),
            "git commit",
        )?;
        run_command(
            Command::new("git").args([
                "-C",
                work.to_string_lossy().as_ref(),
                "remote",
                "add",
                "origin",
                bare.to_string_lossy().as_ref(),
            ]),
            "git remote add",
        )?;
        run_command(
            Command::new("git").args(["-C", work.to_string_lossy().as_ref(), "push", "origin", "HEAD:master"]),
            "git push master",
        )?;

        Ok(bare)
    }

    #[test]
    fn post_receive_requires_existing_build_workspace() -> Result<()> {
        let root = temp_dir_path("build_workspace_missing");
        fs::create_dir_all(&root)?;

        let bare = create_remote_with_master_commit(&root)?;
        let deploy_root = root.join("deploy");
        let config_path = root.join("bones.yaml");
        write_config(&config_path, &bare, &deploy_root, "master")?;

        let result = run(config_path.to_string_lossy().as_ref(), None);
        assert!(result.is_err());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn post_receive_checks_out_requested_revision_into_build_workspace() -> Result<()> {
        let root = temp_dir_path("checkout_revision");
        fs::create_dir_all(&root)?;

        let bare = create_remote_with_master_commit(&root)?;
        let deploy_root = root.join("deploy");
        let build_root = deploy_root.join(Constants::BUILD_DIR).join(Constants::BUILD_WORKSPACE_DIR);
        fs::create_dir_all(&build_root)?;

        let config_path = root.join("bones.yaml");
        write_config(&config_path, &bare, &deploy_root, "master")?;

        let config_yaml_path = config_path.to_string_lossy().to_string();
        let cfg = crate::config::load(Path::new(&config_yaml_path))?;
        release_state::write_staged_release(&cfg, "20260507_181010")?;

        let result = run(&config_yaml_path, Some("master"));

        // Documented behavior: post-receive checks out code as deploy user and succeeds;
        // shared wiring is performed as a separate privileged step.
        assert!(result.is_ok());
        assert!(build_root.join("README.md").exists());

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
