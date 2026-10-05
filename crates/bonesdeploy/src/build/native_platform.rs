use std::ffi::OsString;
use std::path::Path;

#[cfg(unix)]
use anyhow::Result;

#[cfg(unix)]
use anyhow::Context;
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[cfg(unix)]
#[derive(Clone, Copy)]
pub(super) struct MountOwnership {
    pub(super) uid: u32,
    pub(super) gid: u32,
}

#[cfg(unix)]
pub(super) fn mount_ownership(path: &Path) -> Result<MountOwnership> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect local build mount {}", path.display()))?;
    Ok(MountOwnership { uid: metadata.uid(), gid: metadata.gid() })
}

#[cfg(unix)]
pub(super) fn set_private_environment_file_permissions(path: &Path) -> Result<()> {
    fs::set_permissions(path, PermissionsExt::from_mode(0o600)).context("Failed to protect build environment file")
}

#[cfg(windows)]
pub(super) fn set_private_environment_file_permissions(_path: &Path) {}

/// Constructs one Docker bind argument without parsing or rewriting host paths.
pub(super) fn bind_specification(host: &Path, container: &str, access: &str) -> OsString {
    let mut specification = host.as_os_str().to_os_string();
    specification.push(":");
    specification.push(container);
    specification.push(access);
    specification
}
