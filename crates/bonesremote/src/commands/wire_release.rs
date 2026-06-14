use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::release_state;

use shared::config::{Shared, SharedPath};

pub fn run(config_path: &str) -> Result<()> {
    let config_path = Path::new(config_path);
    let cfg = config::load(config_path)?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let build_root = release_state::build_root(&cfg);
    let shared_dir = release_state::shared_dir(&cfg);

    let shared_paths = load_runtime_shared_paths(config_path)?;
    for shared_path in &shared_paths {
        validate_shared_path(&shared_path.path)?;
        wire_path(&build_root, &shared_dir, &shared_path.path)?;
    }

    println!("Wired build workspace for staged release: {release_name}");
    Ok(())
}

fn load_runtime_shared_paths(config_path: &Path) -> Result<Vec<SharedPath>> {
    #[derive(serde::Deserialize)]
    struct RuntimeShared {
        #[serde(default)]
        shared: Shared,
    }
    let runtime_path = config_path.parent().unwrap_or(Path::new(".")).join("runtime.yaml");
    if !runtime_path.exists() {
        return Ok(Vec::new());
    }
    let content =
        fs::read_to_string(&runtime_path).with_context(|| format!("Failed to read {}", runtime_path.display()))?;
    let rt: RuntimeShared =
        serde_yml::from_str(&content).with_context(|| format!("Failed to parse {}", runtime_path.display()))?;
    Ok(rt.shared.paths)
}

fn validate_shared_path(relative_path: &str) -> Result<()> {
    if relative_path.is_empty() {
        bail!("shared path must not be empty");
    }
    if relative_path.starts_with('/') {
        bail!("shared path must be relative, got: {relative_path}");
    }
    if relative_path.contains("..") {
        bail!("shared path must not contain .., got: {relative_path}");
    }
    Ok(())
}

fn wire_path(build_root: &Path, shared_dir: &Path, relative_path: &str) -> Result<()> {
    let release_path = build_root.join(relative_path);
    let shared_path = shared_dir.join(relative_path);

    if !shared_path_exists(&shared_path) {
        bail!("shared path does not exist: {}", shared_path.display());
    }

    ensure_parent_exists(&release_path)?;
    if release_path_is_resolved(&release_path) {
        remove_path(&release_path)?;
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

fn shared_path_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn release_path_is_resolved(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn remove_path(path: &Path) -> Result<()> {
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

    use super::{remove_path, validate_shared_path};

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_wire_release_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    /// Removes both files and directories, including nested contents.
    #[test]
    fn remove_path_removes_files_and_directories() -> Result<()> {
        let root = temp_dir_path("remove_path");
        fs::create_dir_all(&root)?;

        let file_path = root.join("tmp.txt");
        fs::write(&file_path, "payload")?;
        remove_path(&file_path)?;
        assert!(!file_path.exists());

        let dir_path = root.join("tmp_dir");
        fs::create_dir_all(dir_path.join("nested"))?;
        fs::write(dir_path.join("nested").join("file.txt"), "payload")?;
        remove_path(&dir_path)?;
        assert!(!dir_path.exists());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Rejects empty, absolute, and parent-directory paths.
    #[test]
    fn validate_shared_path_rejects_unsafe_paths() {
        assert!(validate_shared_path("").is_err());
        assert!(validate_shared_path("/etc").is_err());
        assert!(validate_shared_path("../.env").is_err());
        assert!(validate_shared_path("storage").is_ok());
        assert!(validate_shared_path("storage/logs").is_ok());
    }
}
