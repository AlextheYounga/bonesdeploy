use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::fs;
use std::io::{Error, ErrorKind};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::ptr::null_mut;

use bonesdeploy_core::paths;

use crate::commands::doctor::security::types::{CurrentState, FileKind, FileNode, PathTree, ReleaseEvidence, Site};

pub fn collect_path_tree(path: &Path, follow_symlink_targets: bool) -> Result<PathTree, String> {
    let mut nodes = BTreeMap::new();
    let mut walked = BTreeSet::new();
    collect_parents(path, &mut nodes)?;
    walk_path(path, follow_symlink_targets, &mut nodes, &mut walked)?;
    Ok(PathTree { requested: path.to_path_buf(), nodes })
}

pub fn collect_release(site: &Site, exhaustive: bool) -> Result<ReleaseEvidence, String> {
    let current_path = site.project_root.join(paths::CURRENT_LINK);
    let releases_path = site.project_root.join(paths::RELEASES_DIR);
    let releases_root = fs::canonicalize(&releases_path)
        .map_err(|error| format!("cannot resolve releases root {}: {error}", releases_path.display()))?;
    let current = match fs::symlink_metadata(&current_path) {
        Ok(metadata) if !metadata.file_type().is_symlink() => CurrentState::NotSymlink,
        Ok(_) => match fs::canonicalize(&current_path) {
            Ok(target) => CurrentState::Active(target),
            Err(error) if error.kind() == ErrorKind::NotFound => CurrentState::Broken,
            Err(error) => return Err(format!("cannot resolve {}: {error}", current_path.display())),
        },
        Err(error) if error.kind() == ErrorKind::NotFound => CurrentState::Missing,
        Err(error) => return Err(format!("cannot inspect {}: {error}", current_path.display())),
    };

    let mut nodes = BTreeMap::new();
    let mut walked = BTreeSet::new();
    collect_parents(&site.project_root, &mut nodes)?;
    collect_node(&site.project_root, &mut nodes)?;
    collect_parents(&releases_root, &mut nodes)?;
    collect_node(&releases_root, &mut nodes)?;
    if !matches!(current, CurrentState::Missing) {
        collect_node(&current_path, &mut nodes)?;
    }
    if let CurrentState::Active(target) = &current {
        collect_parents(target, &mut nodes)?;
        if exhaustive {
            walk_path(target, false, &mut nodes, &mut walked)?;
        } else {
            collect_node(target, &mut nodes)?;
        }
    }

    Ok(ReleaseEvidence {
        site: site.name.clone(),
        releases_root,
        current,
        filesystem: PathTree { requested: current_path, nodes },
    })
}

fn collect_parents(path: &Path, nodes: &mut BTreeMap<PathBuf, FileNode>) -> Result<(), String> {
    let mut cursor = Some(path);
    while let Some(item) = cursor {
        collect_node(item, nodes)?;
        cursor = item.parent();
    }
    Ok(())
}

fn walk_path(
    path: &Path,
    follow_symlink_targets: bool,
    nodes: &mut BTreeMap<PathBuf, FileNode>,
    walked: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    if !walked.insert(path.to_path_buf()) {
        return Ok(());
    }
    let node = collect_node(path, nodes)?.clone();
    if node.kind == FileKind::Symlink {
        if follow_symlink_targets {
            let target =
                fs::canonicalize(path).map_err(|error| format!("cannot resolve {}: {error}", path.display()))?;
            collect_parents(&target, nodes)?;
            walk_path(&target, true, nodes, walked)?;
        }
        return Ok(());
    }
    if node.kind != FileKind::Directory {
        return Ok(());
    }
    for entry in fs::read_dir(path).map_err(|error| format!("cannot read {}: {error}", path.display()))? {
        let entry = entry.map_err(|error| format!("cannot enumerate {}: {error}", path.display()))?;
        walk_path(&entry.path(), follow_symlink_targets, nodes, walked)?;
    }
    Ok(())
}

fn collect_node<'a>(path: &Path, nodes: &'a mut BTreeMap<PathBuf, FileNode>) -> Result<&'a FileNode, String> {
    if !nodes.contains_key(path) {
        let metadata =
            fs::symlink_metadata(path).map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
        let file_type = metadata.file_type();
        let kind = if file_type.is_dir() {
            FileKind::Directory
        } else if file_type.is_file() {
            FileKind::File
        } else if file_type.is_symlink() {
            FileKind::Symlink
        } else {
            FileKind::Other
        };
        let has_acl = has_acl(path, kind)?;
        nodes.insert(
            path.to_path_buf(),
            FileNode {
                path: path.to_path_buf(),
                kind,
                uid: metadata.uid(),
                gid: metadata.gid(),
                mode: metadata.permissions().mode(),
                has_acl,
            },
        );
    }
    nodes.get(path).ok_or_else(|| format!("failed to retain collected metadata for {}", path.display()))
}

fn has_acl(path: &Path, kind: FileKind) -> Result<bool, String> {
    if extended_attribute_exists(path, b"system.posix_acl_access\0")? {
        return Ok(true);
    }
    Ok(kind == FileKind::Directory && extended_attribute_exists(path, b"system.posix_acl_default\0")?)
}

fn extended_attribute_exists(path: &Path, name: &'static [u8]) -> Result<bool, String> {
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| format!("path contains a null byte and cannot be inspected: {}", path.display()))?;
    let name = name.as_ptr().cast();
    // SAFETY: both pointers reference NUL-terminated storage for the duration
    // of the call. A null value buffer with size zero asks only for its length.
    let result = unsafe { libc::lgetxattr(path.as_ptr(), name, null_mut(), 0) };
    if result >= 0 {
        return Ok(true);
    }
    let error = Error::last_os_error();
    match error.raw_os_error() {
        Some(code) if code == libc::ENODATA || code == libc::ENOTSUP => Ok(false),
        _ => Err(format!("cannot inspect ACL metadata for {}: {error}", path.to_string_lossy())),
    }
}
