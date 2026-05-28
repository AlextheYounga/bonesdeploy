use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config;
use crate::permissions;
use crate::privileges;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release wire")?;

    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let build_root = release_state::build_root(&cfg);
    let shared_dir = release_state::shared_dir(&cfg);

    for shared_file in &cfg.releases.shared_files {
        wire_path(&cfg, &build_root, &shared_dir, shared_file, true)?;
    }
    for shared_dir_path in &cfg.releases.shared_dirs {
        wire_path(&cfg, &build_root, &shared_dir, shared_dir_path, false)?;
    }

    println!("Wired build workspace for staged release: {release_name}");
    Ok(())
}

fn wire_path(
    cfg: &config::BonesConfig,
    release_dir: &Path,
    shared_dir: &Path,
    relative_path: &str,
    create_file: bool,
) -> Result<()> {
    let release_path = release_dir.join(relative_path);
    let shared_path = shared_dir.join(relative_path);

    if path_exists(&release_path) && !path_exists(&shared_path) {
        ensure_parent_exists(&shared_path)?;
        fs::rename(&release_path, &shared_path).with_context(|| {
            format!(
                "Failed to move release path into shared path: {} -> {}",
                release_path.display(),
                shared_path.display()
            )
        })?;
    }

    if !path_exists(&shared_path) {
        create_default_shared_target(&shared_path, create_file)?;
    }

    permissions::chown_paths_to_deploy_user(cfg, &[shared_path.as_path()], true)?;

    ensure_parent_exists(&release_path)?;
    if path_exists(&release_path) {
        remove_path(&release_path)?;
    }

    symlink(&shared_path, &release_path).with_context(|| {
        format!("Failed to create shared symlink {} -> {}", release_path.display(), shared_path.display())
    })?;

    println!("Linked shared path: {} -> {}", release_path.display(), shared_path.display());

    Ok(())
}

fn create_default_shared_target(shared_path: &Path, create_file: bool) -> Result<()> {
    ensure_parent_exists(shared_path)?;

    if create_file {
        fs::File::create(shared_path)
            .with_context(|| format!("Failed to create shared file: {}", shared_path.display()))?;
    } else {
        fs::create_dir_all(shared_path)
            .with_context(|| format!("Failed to create shared directory: {}", shared_path.display()))?;
    }

    Ok(())
}

fn ensure_parent_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }
    Ok(())
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

fn path_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::{create_default_shared_target, remove_path};

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("bonesremote_wire_release_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    // Explicit file declarations must be bootstrapped as files.
    #[test]
    fn create_default_shared_target_creates_file_for_explicit_file_paths() -> Result<()> {
        let root = temp_dir_path("default_file");
        let shared_file = root.join("shared").join(".env");

        create_default_shared_target(&shared_file, true)?;

        assert!(shared_file.exists());
        assert!(shared_file.is_file());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    // Explicit directory declarations must be bootstrapped as directories.
    #[test]
    fn create_default_shared_target_creates_directory_for_explicit_directory_paths() -> Result<()> {
        let root = temp_dir_path("default_directory");
        let shared_dir = root.join("shared").join("storage");

        create_default_shared_target(&shared_dir, false)?;

        assert!(shared_dir.exists());
        assert!(shared_dir.is_dir());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    // Verifies cleanup helper removes both file and directory paths before relinking.
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
}
