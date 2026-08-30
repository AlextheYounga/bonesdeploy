use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
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
pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().with_context(|| format!("State file {} has no parent", path.display()))?;
    if !parent.is_dir() {
        anyhow::bail!(
            "Deployment state directory {} is missing. Run 'bonesdeploy site setup' to provision it.",
            parent.display()
        );
    }

    let file_name = path.file_name().with_context(|| format!("State file {} has no file name", path.display()))?;
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).context("System clock is before UNIX_EPOCH")?.as_nanos();
    let temp = parent.join(format!(".{}.tmp-{}-{nanos}", file_name.to_string_lossy(), process::id()));

    {
        let mut file =
            File::create(&temp).with_context(|| format!("Failed to create temporary state file {}", temp.display()))?;
        file.write_all(contents).with_context(|| format!("Failed to write temporary state file {}", temp.display()))?;
        file.flush().with_context(|| format!("Failed to flush temporary state file {}", temp.display()))?;
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Failed to set permissions on temporary state file {}", temp.display()))?;
        file.sync_all().with_context(|| format!("Failed to sync temporary state file {}", temp.display()))?;
    }

    fs::rename(&temp, path).with_context(|| format!("Failed to atomically replace state file {}", path.display()))?;

    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .with_context(|| format!("Failed to sync state directory {}", parent.display()))?;

    Ok(())
}
