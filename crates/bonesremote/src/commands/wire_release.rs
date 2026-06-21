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
        for shared_path in &shared_paths {
            validate_shared_path(&shared_path.path)?;
            wire_path(&build_root, &shared_dir, &shared_path.path)?;
        }
        println!("Wired {} shared path(s) for staged release: {release_name}", shared_paths.len());
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
    /// nested under a `[shared]` table, not at the TOML root.
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
        assert_eq!(config.shared.paths[1].path, "storage");
        assert_eq!(config.shared.paths[1].path_type, PathType::Dir);
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
