use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::release_state;

pub fn run(config_path: &str, revision: Option<&str>) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let build_root = release_state::build_root(&cfg);
    ensure_build_workspace_accessible(&build_root)?;

    let checkout_target = revision.unwrap_or(cfg.data.branch.as_str());
    println!("Checking out {checkout_target} to {}...", build_root.display());

    let status = Command::new("git")
        .arg("--work-tree")
        .arg(&build_root)
        .arg("--git-dir")
        .arg(&cfg.data.repo_path)
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

    Ok(())
}

fn ensure_build_workspace_accessible(build_root: &Path) -> Result<()> {
    match fs::metadata(build_root) {
        Ok(metadata) => {
            if metadata.is_dir() {
                Ok(())
            } else {
                bail!("Build workspace is not a directory: {}", build_root.display())
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!("Build workspace does not exist: {}", build_root.display())
        }
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            bail!("Build workspace is not accessible (permission denied): {}", build_root.display())
        }
        Err(error) => bail!("Failed to inspect build workspace {}: {error}", build_root.display()),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;

    use super::run;
    use crate::config::Constants;

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_post_receive_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    fn run_command(command: &mut Command, label: &str) -> Result<()> {
        let status = command.status()?;
        anyhow::ensure!(status.success(), "Command failed ({label}) with status {status}");
        Ok(())
    }

    fn write_config(path: &Path, repo_path: &Path, project_root: &Path, branch: &str) -> Result<()> {
        let cfg = crate::config::BonesConfig {
            data: crate::config::Data {
                remote_name: String::from("production"),
                project_name: String::from("postreceive"),
                host: String::from("localhost"),
                port: String::from("22"),
                repo_path: repo_path.to_string_lossy().to_string(),
                project_root: project_root.to_string_lossy().to_string(),
                web_root: String::from("public"),
                branch: branch.to_string(),
                deploy_on_push: true,
            },
            releases: crate::config::Releases { keep: 5 },
        };
        let yaml = serde_yml::to_string(&cfg)?;
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

    /// `post-receive` fails when the build workspace does not exist.
    #[test]
    fn post_receive_requires_existing_build_workspace() -> Result<()> {
        let root = temp_dir_path("build_workspace_missing");
        fs::create_dir_all(&root)?;

        let bare = create_remote_with_master_commit(&root)?;
        let project_root = root.join("deploy");
        let config_path = root.join("bones.yaml");
        write_config(&config_path, &bare, &project_root, "master")?;

        let result = run(config_path.to_string_lossy().as_ref(), None);
        assert!(result.is_err());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// `post-receive` checks out the requested revision into the staged build workspace.
    #[test]
    fn post_receive_checks_out_requested_revision_into_build_workspace() -> Result<()> {
        let root = temp_dir_path("checkout_revision");
        fs::create_dir_all(&root)?;

        let bare = create_remote_with_master_commit(&root)?;
        let project_root = root.join("deploy");
        let build_root = project_root.join(Constants::BUILD_DIR).join(paths::WORKSPACE_DIR);
        fs::create_dir_all(&build_root)?;

        let config_path = root.join("bones.yaml");
        write_config(&config_path, &bare, &project_root, "master")?;

        let config_yaml_path = config_path.to_string_lossy().to_string();
        let result = run(&config_yaml_path, Some("master"));

        // Post-receive is responsible for checkout; shared-path wiring happens in a separate command.
        assert!(result.is_ok());
        assert!(build_root.join("README.md").exists());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// `post-receive` fails with a permission error when the build workspace is inaccessible.
    #[test]
    fn post_receive_reports_permission_denied_for_inaccessible_workspace() -> Result<()> {
        let root = temp_dir_path("workspace_permission_denied");
        fs::create_dir_all(&root)?;

        let bare = create_remote_with_master_commit(&root)?;
        let project_root = root.join("deploy");
        let build_dir = project_root.join(Constants::BUILD_DIR);
        let build_root = build_dir.join(paths::WORKSPACE_DIR);
        fs::create_dir_all(&build_root)?;

        let config_path = root.join("bones.yaml");
        write_config(&config_path, &bare, &project_root, "master")?;

        let mut perms = fs::metadata(&build_dir)?.permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&build_dir, perms)?;

        let result = run(config_path.to_string_lossy().as_ref(), Some("master"));

        let mut restore = fs::metadata(&build_dir)?.permissions();
        restore.set_mode(0o755);
        fs::set_permissions(&build_dir, restore)?;

        let Err(error) = result else {
            anyhow::bail!("post-receive should fail when workspace path is inaccessible");
        };
        assert!(error.to_string().to_lowercase().contains("permission denied"));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
