use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::ownership;

pub(super) fn prepare_release_tree(
    source: &Path,
    destination: &Path,
    runtime_user: &str,
    release_group: &str,
) -> Result<()> {
    if !source.is_dir() {
        bail!("Source tree is not a directory: {}", source.display());
    }

    fs::create_dir_all(destination)
        .with_context(|| format!("Failed to create release directory {}", destination.display()))?;
    clear_directory_children(destination)?;

    copy_hardened(source, destination, source)?;
    set_release_tree_owner(destination, ownership::user_uid(runtime_user)?, release_group)?;
    Ok(())
}

pub(super) fn seal_release_tree(destination: &Path, release_group: &str) -> Result<()> {
    set_release_tree_owner(destination, root_uid()?, release_group)
}

fn copy_hardened(source: &Path, destination: &Path, tree_root: &Path) -> Result<()> {
    for entry in fs::read_dir(source).with_context(|| format!("Failed to read source tree {}", source.display()))? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)
            .with_context(|| format!("Failed to inspect build artifact {}", source_path.display()))?;
        let file_type = metadata.file_type();

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path)
                .with_context(|| format!("Failed to create release directory {}", dest_path.display()))?;
            copy_hardened(&source_path, &dest_path, tree_root)?;
            continue;
        }

        if file_type.is_file() {
            fs::copy(&source_path, &dest_path).with_context(|| {
                format!("Failed to copy build artifact {} into {}", source_path.display(), dest_path.display())
            })?;
            continue;
        }

        if file_type.is_symlink() {
            let target = fs::read_link(&source_path)
                .with_context(|| format!("Failed to read symlink {}", source_path.display()))?;
            validate_symlink_target(&source_path, &target, tree_root)?;
            symlink(&target, &dest_path)
                .with_context(|| format!("Failed to recreate symlink {}", dest_path.display()))?;
            continue;
        }

        bail!("Unsupported artifact type in promoted release: {}", source_path.display());
    }

    Ok(())
}

pub(super) fn validate_symlink_target(link_path: &Path, target: &Path, tree_root: &Path) -> Result<()> {
    if target.is_absolute() {
        bail!("Absolute symlink is not allowed in release artifacts: {} -> {}", link_path.display(), target.display());
    }

    let link_parent = link_path.parent().unwrap_or(tree_root);
    let candidate = normalize_relative_path(&link_parent.join(target), tree_root)?;
    if !candidate.starts_with(tree_root) {
        bail!("Symlink escapes release tree: {} -> {}", link_path.display(), target.display());
    }

    Ok(())
}

pub(super) fn normalize_relative_path(path: &Path, root: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized == root || !normalized.pop() {
                    bail!("Path escapes release tree: {}", path.display());
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Ok(normalized)
}

fn set_release_tree_owner(destination: &Path, uid: u32, release_group: &str) -> Result<()> {
    let gid = site_group_gid(release_group)?;
    set_release_tree_identity(destination, uid, gid)
}

fn set_release_tree_identity(destination: &Path, uid: u32, gid: u32) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::chown;
    let metadata = fs::symlink_metadata(destination)
        .with_context(|| format!("Failed to inspect {} for sealing", destination.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    chown(destination, Some(uid), Some(gid)).with_context(|| format!("Failed to chown {}", destination.display()))?;

    let mode = if metadata.file_type().is_dir() {
        0o750
    } else if metadata.mode() & 0o111 != 0 {
        0o750
    } else {
        0o640
    };
    fs::set_permissions(destination, fs::Permissions::from_mode(mode))
        .with_context(|| format!("Failed to set permissions on {}", destination.display()))?;

    if metadata.file_type().is_dir() {
        for entry in fs::read_dir(destination)
            .with_context(|| format!("Failed to read {} for sealing", destination.display()))?
        {
            let entry = entry?;
            set_release_tree_identity(&entry.path(), uid, gid)?;
        }
    }

    Ok(())
}

fn root_uid() -> Result<u32> {
    super::ownership::user_uid("root")
}

fn site_group_gid(group: &str) -> Result<u32> {
    super::ownership::site_group_gid(group)
}

fn clear_directory_children(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("Failed to read release directory {}", path.display()))? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        if entry_type.is_dir() {
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::Path;
    use std::process;

    use anyhow::Result;

    use super::{
        clear_directory_children, normalize_relative_path, set_release_tree_identity, validate_symlink_target,
    };

    #[test]
    fn clear_directory_children_only_removes_entries() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-promote-clear-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join("file.txt"), "x")?;
        fs::create_dir_all(root.join("nested"))?;

        clear_directory_children(&root)?;

        assert!(root.exists(), "clear must not remove the directory itself");
        assert!(fs::read_dir(&root)?.next().is_none());

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn candidate_tree_is_writable_by_its_temporary_owner() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-promote-writable-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        let public = root.join("public");
        fs::create_dir_all(&public)?;
        fs::write(public.join("index.php"), "<?php")?;

        let metadata = fs::metadata(&root)?;
        set_release_tree_identity(&root, metadata.uid(), metadata.gid())?;

        assert_eq!(fs::metadata(&public)?.permissions().mode() & 0o777, 0o750);
        assert_eq!(fs::metadata(public.join("index.php"))?.permissions().mode() & 0o777, 0o640);

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn normalize_relative_path_rejects_escape() {
        let root = Path::new("/tmp/release-root");
        let escaped = normalize_relative_path(Path::new("/tmp/release-root/app/../../etc/passwd"), root);
        assert!(escaped.is_err());
    }

    #[test]
    fn validate_symlink_target_rejects_absolute_and_escaping_targets() {
        let root = Path::new("/tmp/release-root");
        assert!(validate_symlink_target(Path::new("/tmp/release-root/x"), Path::new("/etc/passwd"), root).is_err());
        assert!(
            validate_symlink_target(Path::new("/tmp/release-root/public/x"), Path::new("../../evil"), root).is_err()
        );
    }
}
