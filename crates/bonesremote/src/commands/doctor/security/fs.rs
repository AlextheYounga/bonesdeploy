use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use super::types::Account;

pub(super) fn writable_in_path_chain<'a>(
    path: &Path,
    accounts: &'a [&Account],
) -> Result<Option<(&'a Account, PathBuf)>, String> {
    let mut chain = Vec::new();
    let mut cursor = Some(path);
    while let Some(item) = cursor {
        chain.push(item.to_path_buf());
        cursor = item.parent();
    }
    for item in chain {
        for account in accounts {
            if account_can_write(&item, account)? {
                return Ok(Some((account, item)));
            }
        }
    }
    Ok(None)
}

pub(super) fn find_runtime_writable(path: &Path, account: &Account) -> Result<Option<PathBuf>, String> {
    if account_can_write(path, account)?
        || fs::symlink_metadata(path).map_err(|error| format!("cannot inspect {}: {error}", path.display()))?.uid()
            == account.uid
    {
        return Ok(Some(path.to_path_buf()));
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    if !metadata.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(path).map_err(|error| format!("cannot read {}: {error}", path.display()))? {
        let entry = entry.map_err(|error| format!("cannot enumerate {}: {error}", path.display()))?;
        if let Some(writable) = find_runtime_writable(&entry.path(), account)? {
            return Ok(Some(writable));
        }
    }
    Ok(None)
}

pub(super) fn account_can_write(path: &Path, account: &Account) -> Result<bool, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    let mode = metadata.permissions().mode();
    Ok(if metadata.uid() == account.uid {
        mode & 0o200 != 0
    } else if account.groups.contains(&metadata.gid()) {
        mode & 0o020 != 0
    } else {
        mode & 0o002 != 0
    })
}

pub(super) fn has_login_shell(shell: &str) -> bool {
    !matches!(shell, "/usr/sbin/nologin" | "/sbin/nologin" | "/bin/false")
}

// Resolves a symlink, returning None if the target does not exist.
pub(super) fn try_canonicalize(path: &Path) -> Result<Option<PathBuf>, String> {
    match fs::canonicalize(path) {
        Ok(target) => Ok(Some(target)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("cannot resolve {}: {error}", path.display())),
    }
}
