use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config;
use crate::release_state;
use shared::paths;

pub fn run(config_path: &str) -> Result<()> {
    let config_path = Path::new(config_path);
    let cfg = config::load(config_path)?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let build_root = PathBuf::from(cfg.deployment_paths(paths::DEFAULT_WEB_ROOT).build_root);
    let shared_dir = PathBuf::from(cfg.deployment_paths(paths::DEFAULT_WEB_ROOT).shared);

    let shared_env = shared_dir.join(".env");
    let workspace_env = build_root.join(".env");
    let workspace_env_example = build_root.join(".env.example");

    if shared_env.exists() {
        remove_workspace_path(&workspace_env)?;
        symlink(&shared_env, &workspace_env)
            .with_context(|| format!("Failed to link {} -> {}", workspace_env.display(), shared_env.display()))?;
        println!("Linked .env for {}: {} -> {}", release_name, workspace_env.display(), shared_env.display());
    } else if workspace_env_example.exists() {
        fs::create_dir_all(&shared_dir)
            .with_context(|| format!("Failed to create shared dir: {}", shared_dir.display()))?;
        fs::copy(&workspace_env_example, &shared_env).with_context(|| {
            format!("Failed to copy {} to {}", workspace_env_example.display(), shared_env.display())
        })?;
        fs::set_permissions(&shared_env, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Failed to set permissions on {}", shared_env.display()))?;
        remove_workspace_path(&workspace_env)?;
        symlink(&shared_env, &workspace_env)
            .with_context(|| format!("Failed to link {} -> {}", workspace_env.display(), shared_env.display()))?;
        println!(
            "Created shared .env from .env.example and linked it for {}:\n{} -> {}",
            release_name,
            workspace_env.display(),
            shared_env.display()
        );
    } else {
        println!(
            "No shared .env or .env.example found for {release_name}; skipping .env link. Use bonesdeploy secrets to provide .env."
        );
    }

    Ok(())
}

fn remove_workspace_path(path: &Path) -> Result<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };

    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove directory {}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::{Result, bail};

    use super::remove_workspace_path;

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_wire_release_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    fn write_bones_toml(root: &Path, project_root: &Path, repo_path: &Path) -> Result<PathBuf> {
        let bones_toml = root.join("bones.toml");
        let content = format!(
            r#"
project_name = "testapp"
host = "example.com"
repo_path = "{}"
project_root = "{}"
"#,
            repo_path.display(),
            project_root.display()
        );
        fs::write(&bones_toml, content)?;
        Ok(bones_toml)
    }

    fn write_staged_release(repo_path: &Path, release_name: &str) -> Result<()> {
        let staged_file = repo_path.join("bones").join(".staged_release");
        let Some(parent) = staged_file.parent() else {
            bail!("staged release file has no parent directory");
        };
        fs::create_dir_all(parent)?;
        fs::write(&staged_file, format!("{release_name}\n"))?;
        Ok(())
    }

    #[test]
    fn remove_workspace_path_removes_files_and_directories() -> Result<()> {
        let root = temp_dir_path("remove_workspace_path");
        fs::create_dir_all(&root)?;

        let file_path = root.join("tmp.txt");
        fs::write(&file_path, "payload")?;
        remove_workspace_path(&file_path)?;
        assert!(!file_path.exists());

        let dir_path = root.join("tmp_dir");
        fs::create_dir_all(dir_path.join("nested"))?;
        fs::write(dir_path.join("nested").join("file.txt"), "payload")?;
        remove_workspace_path(&dir_path)?;
        assert!(!dir_path.exists());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn existing_shared_env_is_linked_into_workspace() -> Result<()> {
        let root = temp_dir_path("existing_shared_env");
        let project_root = root.join("deploy");
        let repo_path = root.join("repo.git");
        let build_workspace = project_root.join("build").join("workspace");
        let shared_dir = project_root.join("shared");

        fs::create_dir_all(&build_workspace)?;
        fs::create_dir_all(&shared_dir)?;
        fs::write(shared_dir.join(".env"), "APP_ENV=production\n")?;
        fs::write(build_workspace.join(".env"), "APP_ENV=local\n")?;

        let bones_toml_path = write_bones_toml(&root, &project_root, &repo_path)?;
        write_staged_release(&repo_path, "20250101_000000")?;

        super::run(&bones_toml_path.to_string_lossy())?;

        let link = build_workspace.join(".env");
        assert!(link.is_symlink(), "expected .env to be a symlink");
        assert_eq!(fs::read_link(&link)?, shared_dir.join(".env"));
        assert_eq!(fs::read_to_string(&link)?, "APP_ENV=production\n");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn missing_shared_env_with_env_example_copies_and_links() -> Result<()> {
        let root = temp_dir_path("missing_shared_env");
        let project_root = root.join("deploy");
        let repo_path = root.join("repo.git");
        let build_workspace = project_root.join("build").join("workspace");
        let shared_dir = project_root.join("shared");

        fs::create_dir_all(&build_workspace)?;
        fs::create_dir_all(&shared_dir)?;
        fs::write(build_workspace.join(".env.example"), "APP_ENV=production\n")?;

        let bones_toml_path = write_bones_toml(&root, &project_root, &repo_path)?;
        write_staged_release(&repo_path, "20250101_000000")?;

        super::run(&bones_toml_path.to_string_lossy())?;

        let shared_env = shared_dir.join(".env");
        assert!(shared_env.exists(), "shared .env should be created from .env.example");
        assert_eq!(fs::read_to_string(&shared_env)?, "APP_ENV=production\n");

        let link = build_workspace.join(".env");
        assert!(link.is_symlink(), "expected .env to be a symlink");
        assert_eq!(fs::read_link(&link)?, shared_env);

        let metadata = fs::symlink_metadata(&shared_env)?;
        let mode = metadata.permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "shared .env should have 600 permissions");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn existing_shared_env_is_not_overwritten_by_env_example() -> Result<()> {
        let root = temp_dir_path("shared_env_not_overwritten");
        let project_root = root.join("deploy");
        let repo_path = root.join("repo.git");
        let build_workspace = project_root.join("build").join("workspace");
        let shared_dir = project_root.join("shared");

        fs::create_dir_all(&build_workspace)?;
        fs::create_dir_all(&shared_dir)?;
        fs::write(shared_dir.join(".env"), "APP_ENV=staging\n")?;
        fs::write(build_workspace.join(".env.example"), "APP_ENV=production\n")?;

        let bones_toml_path = write_bones_toml(&root, &project_root, &repo_path)?;
        write_staged_release(&repo_path, "20250101_000000")?;

        super::run(&bones_toml_path.to_string_lossy())?;

        assert_eq!(fs::read_to_string(shared_dir.join(".env"))?, "APP_ENV=staging\n");
        let link = build_workspace.join(".env");
        assert!(link.is_symlink());
        assert_eq!(fs::read_link(&link)?, shared_dir.join(".env"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn missing_both_env_and_example_does_not_fail() -> Result<()> {
        let root = temp_dir_path("missing_both_env");
        let project_root = root.join("deploy");
        let repo_path = root.join("repo.git");
        let build_workspace = project_root.join("build").join("workspace");
        let shared_dir = project_root.join("shared");

        fs::create_dir_all(&build_workspace)?;
        fs::create_dir_all(&shared_dir)?;

        let bones_toml_path = write_bones_toml(&root, &project_root, &repo_path)?;
        write_staged_release(&repo_path, "20250101_000000")?;

        let result = super::run(&bones_toml_path.to_string_lossy());
        assert!(result.is_ok(), "should not fail when .env and .env.example are missing");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn storage_is_not_symlinked() -> Result<()> {
        let root = temp_dir_path("storage_not_symlinked");
        let project_root = root.join("deploy");
        let repo_path = root.join("repo.git");
        let build_workspace = project_root.join("build").join("workspace");
        let shared_dir = project_root.join("shared");

        fs::create_dir_all(&build_workspace)?;
        fs::create_dir_all(&shared_dir)?;
        fs::create_dir_all(build_workspace.join("storage"))?;
        fs::create_dir_all(shared_dir.join("storage"))?;

        let bones_toml_path = write_bones_toml(&root, &project_root, &repo_path)?;
        write_staged_release(&repo_path, "20250101_000000")?;

        super::run(&bones_toml_path.to_string_lossy())?;

        let storage = build_workspace.join("storage");
        assert!(!storage.is_symlink(), "storage should NOT be symlinked");
        assert!(storage.is_dir(), "storage should still be a directory");

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
