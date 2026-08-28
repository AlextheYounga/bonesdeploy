//! Crash-safe file replacement for managed configuration files.

use std::fs::{self, File, OpenOptions, Permissions};
use std::io::Write;
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Writes `content` to `path` through a temporary file and rename, preserving
/// the existing file mode when the file already exists.
///
/// # Errors
/// Returns an error when the temporary file cannot be created, written, or
/// moved into place.
pub fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let dir = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let temp = dir.join(format!(".bones-{}-{}", process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
    let result = write_via_temp(path, &temp, content);
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.with_context(|| format!("Failed to atomically write {}", path.display()))
}

fn write_via_temp(path: &Path, temp: &Path, content: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(temp).with_context(|| format!("Failed to create temporary file {}", temp.display()))?;
    if let Ok(metadata) = fs::metadata(path) {
        preserve_mode(&file, &metadata.permissions());
    }
    file.write_all(content).context("Failed to write temporary file")?;
    file.sync_all().context("Failed to sync temporary file")?;
    fs::rename(temp, path).with_context(|| format!("Failed to replace {}", path.display()))?;
    let dir = temp.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    File::open(dir)
        .context("Failed to open target directory")?
        .sync_all()
        .context("Failed to sync target directory")?;
    Ok(())
}

#[cfg(unix)]
fn preserve_mode(file: &File, permissions: &Permissions) {
    use std::os::unix::fs::PermissionsExt;
    let _ = file.set_permissions(Permissions::from_mode(permissions.mode()));
}

#[cfg(not(unix))]
fn preserve_mode(_file: &File, _permissions: &Permissions) {}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn atomic_write_replaces_the_target_and_preserves_its_mode() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("managed.env");
        fs::write(&path, "old")?;
        #[cfg(unix)]
        fs::set_permissions(&path, Permissions::from_mode(0o600))?;

        atomic_write(&path, b"new")?;

        assert_eq!(fs::read_to_string(&path)?, "new");
        #[cfg(unix)]
        assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);
        Ok(())
    }

    #[test]
    fn atomic_write_creates_a_missing_target() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("missing.env");

        atomic_write(&path, b"content")?;

        assert_eq!(fs::read_to_string(&path)?, "content");
        Ok(())
    }

    #[test]
    fn atomic_write_leaves_no_temporary_files_behind() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("managed.env");

        atomic_write(&path, b"content")?;

        let leftovers: Vec<_> = fs::read_dir(dir.path())?.filter_map(|entry| entry.ok()).collect();
        assert_eq!(leftovers.len(), 1, "only the target file should remain");
        Ok(())
    }
}
