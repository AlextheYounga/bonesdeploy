use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::config as shared_config;
use shared::paths;

use crate::config;
use crate::release_state;

use shared::config::Shared;

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default)]
struct RuntimeSharedConfig {
    shared: Shared,
}

pub fn run(config_path: &str) -> Result<()> {
    let config_path = Path::new(config_path);
    let cfg = config::load(config_path)?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let build_root = PathBuf::from(cfg.deployment_paths(paths::DEFAULT_WEB_ROOT).build_root);
    let shared_dir = PathBuf::from(cfg.deployment_paths(paths::DEFAULT_WEB_ROOT).shared);

    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    let _runtime = shared_config::load_runtime(config_dir)?;
    let runtime_path = config_dir.join("runtime.toml");
    let shared_paths = if runtime_path.exists() {
        let content =
            fs::read_to_string(&runtime_path).with_context(|| format!("Failed to read {}", runtime_path.display()))?;
        let runtime: RuntimeSharedConfig =
            toml::from_str(&content).with_context(|| format!("Failed to parse {}", runtime_path.display()))?;
        runtime.shared.paths
    } else {
        Vec::new()
    };
    if shared_paths.is_empty() {
        println!("No shared paths configured. Nothing to wire for staged release: {release_name}");
    } else {
        let mut wired = 0usize;
        let mut provisioned = 0usize;
        for shared_path in &shared_paths {
            validate_shared_path(&shared_path.path)?;
            if shared_path.link {
                wire_path(&build_root, &shared_dir, &shared_path.path)?;
                wired += 1;
            } else {
                println!("Shared path is provision-only, not linked: {}", shared_path.path);
                provisioned += 1;
            }
        }
        let parts: Vec<String> = {
            let mut p = Vec::new();
            if wired > 0 {
                p.push(format!("{wired} linked"));
            }
            if provisioned > 0 {
                p.push(format!("{provisioned} provision-only"));
            }
            p
        };
        println!("Shared paths for staged release {release_name}: {}", parts.join(", "));
    }

    Ok(())
}

fn validate_shared_path(relative_path: &str) -> Result<()> {
    if relative_path.is_empty() {
        bail!("shared path must not be empty");
    }
    for component in Path::new(relative_path).components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => bail!("shared path must not contain ."),
            Component::ParentDir => bail!("shared path must not contain .., got: {relative_path}"),
            Component::RootDir | Component::Prefix(_) => {
                bail!("shared path must be relative, got: {relative_path}")
            }
        }
    }
    Ok(())
}

fn wire_path(build_root: &Path, shared_dir: &Path, relative_path: &str) -> Result<()> {
    let release_path = build_root.join(relative_path);
    let shared_path = shared_dir.join(relative_path);

    if fs::symlink_metadata(&shared_path).is_err() {
        bail!(
            "shared path does not exist: {}. Provision it first with 'bonesdeploy remote setup'.",
            shared_path.display()
        );
    }

    ensure_parent_exists(&release_path)?;
    if fs::symlink_metadata(&release_path).is_ok() {
        replace_workspace_path_with_shared_symlink(&release_path)?;
    }

    symlink(&shared_path, &release_path).with_context(|| {
        format!("Failed to create shared symlink {} -> {}", release_path.display(), shared_path.display())
    })?;

    println!("Linked shared path: {} -> {}", release_path.display(), shared_path.display());

    Ok(())
}

fn ensure_parent_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }
    Ok(())
}

/// Removes whatever exists at the build workspace path (file, dir, or symlink) so
/// a shared-path symlink can replace it. Only safe to call against the disposable
/// build workspace — never against `current`, `releases/`, or `shared/`.
fn replace_workspace_path_with_shared_symlink(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect path for removal: {}", path.display()))?;

    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).with_context(|| format!("Failed to remove path: {}", path.display()))?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove directory: {}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::{RuntimeSharedConfig, replace_workspace_path_with_shared_symlink, validate_shared_path, wire_path};
    use shared::config::PathType;

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_wire_release_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    /// Removes both files and directories, including nested contents, from the build workspace.
    #[test]
    fn replace_workspace_path_removes_files_and_directories() -> Result<()> {
        let root = temp_dir_path("replace_workspace_path");
        fs::create_dir_all(&root)?;

        let file_path = root.join("tmp.txt");
        fs::write(&file_path, "payload")?;
        replace_workspace_path_with_shared_symlink(&file_path)?;
        assert!(!file_path.exists());

        let dir_path = root.join("tmp_dir");
        fs::create_dir_all(dir_path.join("nested"))?;
        fs::write(dir_path.join("nested").join("file.txt"), "payload")?;
        replace_workspace_path_with_shared_symlink(&dir_path)?;
        assert!(!dir_path.exists());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Rejects empty, absolute, and parent-directory paths.
    /// Allows benign double-dots in filenames (e.g. "my..dir").
    #[test]
    fn validate_shared_path_rejects_unsafe_paths() {
        assert!(validate_shared_path("").is_err());
        assert!(validate_shared_path("/etc").is_err());
        assert!(validate_shared_path("./.env").is_err(), "explicit current-dir is rejected");
        assert!(validate_shared_path("../.env").is_err());
        assert!(validate_shared_path("storage/../.env").is_err());
        assert!(validate_shared_path("storage").is_ok());
        assert!(validate_shared_path("storage/logs").is_ok());
        assert!(validate_shared_path("my..dir").is_ok(), "double-dot filenames are allowed");
        assert!(validate_shared_path("assets..cache/file.txt").is_ok(), "double-dot directory names are allowed");
    }

    /// Verifies the runtime.toml shape is parsed correctly: `[shared].paths` is
    /// nested under a `[shared]` table, not at the TOML root. Also verifies
    /// `link` defaults to `true` when omitted.
    #[test]
    fn parse_nested_shared_paths_from_runtime_toml() -> Result<()> {
        let toml = r#"
[shared]
paths = [
  { path = ".env", type = "file" },
  { path = "storage", type = "dir" },
]
"#;
        let config: RuntimeSharedConfig = toml::from_str(toml)?;
        assert_eq!(config.shared.paths.len(), 2);
        assert_eq!(config.shared.paths[0].path, ".env");
        assert_eq!(config.shared.paths[0].path_type, PathType::File);
        assert!(config.shared.paths[0].link, "link should default to true");
        assert_eq!(config.shared.paths[1].path, "storage");
        assert_eq!(config.shared.paths[1].path_type, PathType::Dir);
        assert!(config.shared.paths[1].link, "link should default to true");
        Ok(())
    }

    #[test]
    fn parse_link_false_from_runtime_toml() -> Result<()> {
        let toml = r#"
[shared]
paths = [
  { path = "database", type = "dir", link = false },
  { path = "storage", type = "dir", link = true },
]
"#;
        let config: RuntimeSharedConfig = toml::from_str(toml)?;
        assert!(!config.shared.paths[0].link);
        assert!(config.shared.paths[1].link);
        Ok(())
    }

    /// Integration test: calls `run()` with a runtime.toml that has `link =
    /// false` on `database` and verifies it is not symlinked, while `storage`
    /// and `.env` are.
    #[test]
    fn link_false_skipped_by_wire_release() -> Result<()> {
        let root = temp_dir_path("link_false");
        fs::create_dir_all(&root)?;
        let project_root = root.join("deploy");
        let repo_path = root.join("repo.git");
        let build_workspace = project_root.join("build").join("workspace");
        let shared_dir = project_root.join("shared");

        let bones_toml = format!(
            r#"
project_name = "testapp"
host = "example.com"
repo_path = "{}"
project_root = "{}"
"#,
            repo_path.display(),
            project_root.display()
        );
        let bones_toml_path = root.join("bones.toml");
        fs::write(&bones_toml_path, bones_toml)?;

        let runtime_toml = r#"
[shared]
paths = [
  { path = "storage", type = "dir" },
  { path = ".env", type = "file" },
  { path = "database", type = "dir", link = false },
]
"#;
        fs::write(root.join("runtime.toml"), runtime_toml)?;

        let staged_file = repo_path.join("bones").join(".staged_release");
        if let Some(parent) = staged_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&staged_file, "20250101_000000\n")?;

        fs::create_dir_all(&build_workspace)?;
        fs::create_dir_all(build_workspace.join("database").join("migrations"))?;
        fs::write(build_workspace.join("database").join("migrations").join("create_users.php"), "--")?;
        fs::create_dir_all(build_workspace.join("storage"))?;
        fs::write(build_workspace.join(".env"), "APP_NAME=test")?;

        fs::create_dir_all(shared_dir.join("database"))?;
        fs::create_dir_all(shared_dir.join("storage"))?;
        fs::write(shared_dir.join(".env"), "APP_KEY=base64:...")?;

        super::run(&bones_toml_path.to_string_lossy())?;

        let db = build_workspace.join("database");
        assert!(!db.is_symlink(), "database should NOT be a symlink (link=false)");
        assert!(db.is_dir(), "database should still be a directory");
        assert!(db.join("migrations").join("create_users.php").exists(), "database contents preserved");

        let storage_link = build_workspace.join("storage");
        assert!(storage_link.is_symlink(), "storage should be a symlink");
        assert_eq!(fs::read_link(&storage_link)?, shared_dir.join("storage"));

        let env_link = build_workspace.join(".env");
        assert!(env_link.is_symlink(), ".env should be a symlink");
        assert_eq!(fs::read_link(&env_link)?, shared_dir.join(".env"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Verifies that parsing a runtime.toml that has `[shared]` but no `paths`
    /// still succeeds and returns an empty vec.
    #[test]
    fn parse_empty_shared_paths() -> Result<()> {
        let toml = "[shared]\n";
        let config: RuntimeSharedConfig = toml::from_str(toml)?;
        assert!(config.shared.paths.is_empty());
        Ok(())
    }

    #[test]
    fn wire_path_creates_symlink() -> Result<()> {
        let root = temp_dir_path("wire_path");
        let build_root = root.join("build");
        let shared_dir = root.join("shared");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&shared_dir)?;
        fs::write(shared_dir.join(".env"), "SECRET=1")?;

        wire_path(&build_root, &shared_dir, ".env")?;

        let link = build_root.join(".env");
        assert!(link.is_symlink(), "expected symlink at {}", link.display());
        let target = fs::read_link(&link)?;
        assert_eq!(target, shared_dir.join(".env"));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
