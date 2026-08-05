use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

/// Writes `contents` to `path` atomically: a temporary file in the same
/// directory is written, flushed and synced, given the destination's existing
/// permissions, and renamed over the destination; the parent directory is then
/// fsynced. A crash or disk-full condition therefore never leaves truncated
/// deployment state that later status, cancellation, or idle checks cannot
/// parse.
pub(crate) fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().with_context(|| format!("State file {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("Failed to create state directory {}", parent.display()))?;

    let file_name = path.file_name().with_context(|| format!("State file {} has no file name", path.display()))?;
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).context("System clock is before UNIX_EPOCH")?.as_nanos();
    let temp = parent.join(format!(".{}.tmp-{}-{nanos}", file_name.to_string_lossy(), process::id()));

    {
        let mut file =
            File::create(&temp).with_context(|| format!("Failed to create temporary state file {}", temp.display()))?;
        file.write_all(contents).with_context(|| format!("Failed to write temporary state file {}", temp.display()))?;
        file.flush().with_context(|| format!("Failed to flush temporary state file {}", temp.display()))?;
        file.sync_all().with_context(|| format!("Failed to sync temporary state file {}", temp.display()))?;
    }

    if let Ok(metadata) = fs::symlink_metadata(path) {
        fs::set_permissions(&temp, metadata.permissions())
            .with_context(|| format!("Failed to apply permissions to temporary state file {}", temp.display()))?;
    }

    fs::rename(&temp, path).with_context(|| format!("Failed to atomically replace state file {}", path.display()))?;

    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .with_context(|| format!("Failed to sync state directory {}", parent.display()))?;

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
    use bonesdeploy_core::paths;

    use super::atomic_write;

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    #[test]
    fn atomic_write_creates_parent_and_persists_content() -> Result<()> {
        let root = temp_dir("bonesremote_atomic_new")?;
        let target = root.join("nested").join(paths::ACTIVE_DEPLOYMENT_FILE);

        atomic_write(&target, b"{\"phase\":\"building\"}")?;

        assert_eq!(fs::read_to_string(&target)?, "{\"phase\":\"building\"}");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn atomic_write_replaces_existing_content_and_keeps_mode() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("bonesremote_atomic_replace")?;
        let target = root.join(paths::ACTIVE_DEPLOYMENT_FILE);
        fs::write(&target, "old")?;
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600))?;

        atomic_write(&target, b"new")?;

        assert_eq!(fs::read_to_string(&target)?, "new");
        assert_eq!(fs::metadata(&target)?.permissions().mode() & 0o777, 0o600);

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn atomic_write_leaves_no_temporary_file_behind() -> Result<()> {
        let root = temp_dir("bonesremote_atomic_no_tmp")?;
        let target = root.join(paths::STAGED_RELEASE_FILE);
        fs::write(&target, "stale")?;

        atomic_write(&target, b"20260804_190321-46a0b75c-a7f2\n")?;

        let leftovers: Vec<_> = fs::read_dir(&root)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty(), "temporary state files must be renamed away");

        fs::remove_dir_all(root).ok();
        Ok(())
    }
}
