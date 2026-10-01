use std::ffi::CString;
use std::fs::{self, File};
use std::io::Error;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use rustix::fs::{CWD, RenameFlags, renameat_with};

pub fn exchange_directories(left: &Path, right: &Path) -> Result<()> {
    let left_metadata = real_directory_metadata(left)?;
    let right_metadata = real_directory_metadata(right)?;
    if left_metadata.dev() != right_metadata.dev() {
        bail!("Cannot atomically exchange directories on different filesystems");
    }
    renameat_with(CWD, left, CWD, right, RenameFlags::EXCHANGE)
        .context("Atomic shared directory exchange is not supported or failed")
}

pub(super) fn create_transaction_directory(project_root: &Path) -> Result<PathBuf> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).context("System clock is before UNIX_EPOCH")?.as_nanos();
    let path = project_root.join(format!(".shared-import-{}-{nonce}", process::id()));
    fs::create_dir(&path).with_context(|| format!("Failed to create shared import transaction {}", path.display()))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
    Ok(path)
}

pub(super) fn verify_exchange_support(transaction_dir: &Path) -> Result<()> {
    let left = transaction_dir.join("exchange-probe-left");
    let right = transaction_dir.join("exchange-probe-right");
    fs::create_dir(&left)?;
    fs::create_dir(&right)?;
    let result = exchange_directories(&left, &right).and_then(|()| exchange_directories(&left, &right));
    let left_cleanup = fs::remove_dir(&left);
    let right_cleanup = fs::remove_dir(&right);
    result.context("The site filesystem does not support atomic shared directory exchange")?;
    left_cleanup.context("Failed to remove shared import exchange probe")?;
    right_cleanup.context("Failed to remove shared import exchange probe")
}

pub(super) fn real_directory_metadata(path: &Path) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("Failed to inspect {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("Expected a real directory at {}", path.display());
    }
    Ok(metadata)
}

pub(super) fn chown_tree_without_following(path: &Path, uid: u32, gid: u32) -> Result<()> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("Failed to inspect {}", path.display()))?;
    lchown(path, uid, gid)?;
    if metadata.is_dir() {
        for entry in fs::read_dir(path).with_context(|| format!("Failed to read {}", path.display()))? {
            chown_tree_without_following(&entry?.path(), uid, gid)?;
        }
    }
    Ok(())
}

fn lchown(path: &Path, uid: u32, gid: u32) -> Result<()> {
    let path = CString::new(path.as_os_str().as_bytes())
        .with_context(|| format!("Ownership path contains a NUL byte: {}", path.display()))?;
    // SAFETY: `path` is a valid NUL-terminated C string. `lchown` does not
    // retain it and intentionally changes the link itself rather than its target.
    if unsafe { libc::lchown(path.as_ptr(), uid, gid) } != 0 {
        return Err(Error::last_os_error()).context("Failed to set imported shared ownership");
    }
    Ok(())
}

pub(super) fn sync_parent(path: &Path) -> Result<()> {
    let parent = path.parent().context("Shared import path has no parent")?;
    File::open(parent)?.sync_all().with_context(|| format!("Failed to sync {}", parent.display()))
}
