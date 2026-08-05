use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::ownership;

pub(super) fn prepare_release_tree(source: &Path, destination: &Path, runtime_user: &str, group: &str) -> Result<()> {
    if !source.is_dir() {
        bail!("Source tree is not a directory: {}", source.display());
    }

    // Stage creates the release directory exclusively, so it is empty here.
    // Categorically refuse to overwrite a release that already holds content:
    // promotion must never reuse or erase an existing release, possibly the
    // active one.
    ensure_release_dir_empty(destination)?;
    fs::create_dir_all(destination)
        .with_context(|| format!("Failed to create release directory {}", destination.display()))?;

    copy_hardened(source, destination, source)?;
    set_release_tree_owner(destination, ownership::user_uid(runtime_user)?, group)?;
    Ok(())
}

fn ensure_release_dir_empty(destination: &Path) -> Result<()> {
    if destination.exists() && !is_dir_empty(destination)? {
        bail!("Refusing to promote into nonempty release directory {}", destination.display());
    }
    Ok(())
}

pub(super) fn seal_release_tree(destination: &Path, group: &str) -> Result<()> {
    set_release_tree_owner(destination, root_uid()?, group)
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

fn set_release_tree_owner(destination: &Path, uid: u32, group: &str) -> Result<()> {
    let gid = site_group_gid(group)?;
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

    let mode = if metadata.file_type().is_dir() || metadata.mode() & 0o111 != 0 { 0o750 } else { 0o640 };
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

fn is_dir_empty(path: &Path) -> Result<bool> {
    Ok(fs::read_dir(path).with_context(|| format!("Failed to read {}", path.display()))?.next().is_none())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::{
        ensure_release_dir_empty, is_dir_empty, normalize_relative_path, prepare_release_tree,
        set_release_tree_identity, validate_symlink_target,
    };

    #[test]
    fn promote_refuses_nonempty_release_directory() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-promote-nonempty-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        let source = root.join("source");
        let destination = root.join("releases").join("20260804_190321-46a0b75c-0000");
        fs::create_dir_all(source.join("index.html"))?;
        fs::write(source.join("index.html").join("x"), "x")?;
        fs::create_dir_all(&destination)?;
        fs::write(destination.join("existing.txt"), "do-not-touch")?;

        assert!(prepare_release_tree(&source, &destination, "root", "root").is_err());
        assert_eq!(fs::read_to_string(destination.join("existing.txt"))?, "do-not-touch");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn empty_release_directory_passes_the_nonempty_guard() -> Result<()> {
        let root = temp_root("bonesremote-promote-empty");
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        let destination = root.join("releases").join("candidate");
        fs::create_dir_all(&destination)?;

        assert!(is_dir_empty(&destination)?);
        assert!(ensure_release_dir_empty(&destination).is_ok());

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn nonempty_release_directory_fails_the_nonempty_guard() -> Result<()> {
        let root = temp_root("bonesremote-promote-filled");
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        let destination = root.join("releases").join("candidate");
        fs::create_dir_all(&destination)?;
        fs::write(destination.join("file"), "x")?;

        assert!(ensure_release_dir_empty(&destination).is_err());

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    fn temp_root(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos))
    }

    #[test]
    fn candidate_tree_is_writable_by_its_temporary_owner() -> Result<()> {
        let root = temp_root("bonesremote-promote-writable");
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

        fs::remove_dir_all(root).ok();
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
