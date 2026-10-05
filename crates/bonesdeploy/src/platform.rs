#[cfg(unix)]
use std::fs;
use std::fs::{File, OpenOptions};
use std::path::Path;

use anyhow::{Context, Result, bail};

pub(crate) fn installed_bonesdeploy_executable() -> &'static str {
    #[cfg(windows)]
    {
        "bonesdeploy.exe"
    }
    #[cfg(not(windows))]
    {
        "bonesdeploy"
    }
}

#[cfg(unix)]
pub(crate) fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .with_context(|| format!("Failed to set permissions on {}", path.display()))
}

#[cfg(not(unix))]
pub(crate) fn set_mode(_path: &Path, _mode: u32) {}

#[cfg(unix)]
pub(crate) fn sync_parent(path: &Path) -> Result<()> {
    let parent = path.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    File::open(parent)
        .with_context(|| format!("Failed to open directory {}", parent.display()))?
        .sync_all()
        .with_context(|| format!("Failed to sync directory {}", parent.display()))
}

#[cfg(not(unix))]
pub(crate) fn sync_parent(_path: &Path) {}

pub(crate) fn open_regular_file(path: &Path) -> Result<File> {
    #[cfg(unix)]
    let file = {
        use std::os::unix::fs::OpenOptionsExt;

        OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)
    };
    #[cfg(windows)]
    let file: Result<File> = {
        use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
        use windows_sys::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT};

        let file = OpenOptions::new().read(true).custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(path)?;
        if file.metadata()?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            bail!("Archive is a reparse point: {}", path.display());
        }
        Ok(file)
    };
    #[cfg(not(any(unix, windows)))]
    let file = OpenOptions::new().read(true).open(path);

    let file = file.with_context(|| format!("Failed to open archive {}", path.display()))?;
    if !file.metadata()?.is_file() {
        bail!("Archive is not a regular file: {}", path.display());
    }
    Ok(file)
}
