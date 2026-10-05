use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path};
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::collections::{HashMap, HashSet};
#[cfg(windows)]
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

#[cfg(windows)]
use crate::build::inventory::canonical_relative_path;
use crate::build::inventory::{BuildInventory, EntryType, safe_relative_link, validate_path};

pub fn ensure_git_repository() -> Result<()> {
    let output =
        Command::new("git").args(["rev-parse", "--is-inside-work-tree"]).output().context(git_prerequisite())?;

    if !output.status.success() {
        bail!("Not a git repository");
    }

    Ok(())
}

pub fn validate_branch(branch: &str) -> Result<()> {
    let output =
        Command::new("git").args(["check-ref-format", "--branch", branch]).output().context(git_prerequisite())?;
    if !output.status.success() {
        bail!("Invalid Git branch: {branch}");
    }
    Ok(())
}

/// Resolves the configured local branch to an immutable full object ID.
pub fn resolve_branch_commit(repo: &Path, branch: &str) -> Result<String> {
    let reference = paths::branch_ref(branch);
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["rev-parse", "--verify", "--end-of-options", &format!("{reference}^{{commit}}")])
        .output()
        .with_context(|| format!("{} while resolving local branch '{branch}'", git_prerequisite()))?;
    if !output.status.success() {
        bail!("Local branch '{branch}' does not resolve to a commit");
    }

    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("Git returned an invalid commit ID for local branch '{branch}'");
    }
    Ok(revision)
}

/// Exports exactly one committed tree with `git archive`; the worktree is never read.
pub fn export_commit(repo: &Path, revision: &str, destination: &Path) -> Result<BuildInventory> {
    let mut archive = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["archive", "--format=tar", revision])
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("{} while archiving commit {revision}", git_prerequisite()))?;
    let extraction = {
        let archive_stdout = archive.stdout.take().context("Git archive stdout was not piped")?;
        extract_archive(archive_stdout, destination)
    };
    let archive_status = archive.wait().context("Failed to finish Git archive")?;
    if !archive_status.success() {
        bail!("Failed to archive commit {revision}: {archive_status}");
    }
    extraction.with_context(|| format!("Failed to extract commit {revision}"))
}

fn extract_archive<R: Read>(reader: R, destination: &Path) -> Result<BuildInventory> {
    let mut archive = tar::Archive::new(reader);
    let mut inventory = BuildInventory::default();
    let mut symlinks = Vec::new();
    for entry in archive.entries().context("Failed to read Git archive")? {
        let mut entry = entry.context("Failed to read Git archive entry")?;
        let archive_type = entry.header().entry_type();
        if archive_type.is_pax_global_extensions() || archive_type.is_pax_local_extensions() {
            continue;
        }
        let path = entry.path().context("Git archive entry has an invalid path")?.into_owned();
        validate_path(&path)?;
        let entry_type = match archive_type {
            entry_type if entry_type.is_file() => EntryType::File,
            entry_type if entry_type.is_dir() => EntryType::Directory,
            entry_type if entry_type.is_symlink() => EntryType::Symlink,
            entry_type => bail!("Git archive entry {} has unsupported type {entry_type:?}", path.display()),
        };
        let mode = entry.header().mode().context("Git archive entry has an invalid Unix mode")?;
        inventory.record(&path, entry_type, mode)?;
        match entry_type {
            EntryType::Directory => create_directory(destination, &path)?,
            EntryType::File => {
                create_parent_directories(destination, &path)?;
                let output = destination.join(&path);
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&output)
                    .with_context(|| format!("Failed to create Git archive file {}", path.display()))?;
                io::copy(&mut entry, &mut file)
                    .with_context(|| format!("Failed to extract Git archive file {}", path.display()))?;
            }
            EntryType::Symlink => {
                let target = entry
                    .link_name()
                    .context("Failed to read Git archive symlink target")?
                    .context("Git archive symlink has no target")?
                    .into_owned();
                if !safe_relative_link(&path, &target) {
                    bail!("Git archive symlink {} escapes the export destination", path.display());
                }
                symlinks.push((path, target));
            }
        }
    }
    #[cfg(windows)]
    let symlink_targets = symlinks
        .iter()
        .map(|(path, target)| Ok((canonical_relative_path(path)?, target.clone())))
        .collect::<Result<HashMap<_, _>>>()?;
    for (path, target) in symlinks {
        create_parent_directories(destination, &path)?;
        #[cfg(unix)]
        create_symlink(&target, &destination.join(&path))?;
        #[cfg(windows)]
        create_symlink(&target, &destination.join(&path), destination, &symlink_targets)?;
        #[cfg(not(any(unix, windows)))]
        create_symlink(&target, &destination.join(&path))?;
    }
    Ok(inventory)
}

fn create_parent_directories(destination: &Path, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_directory(destination, parent)?;
    }
    Ok(())
}

fn create_directory(destination: &Path, relative: &Path) -> Result<()> {
    let mut current = destination.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            bail!("Git archive directory is not relative and normalized: {}", relative.display());
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => bail!("Git archive path conflicts with non-directory {}", current.display()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&current)
                    .with_context(|| format!("Failed to create Git archive directory {}", current.display()))?;
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Failed to inspect Git archive directory {}", current.display()));
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn create_symlink(target: &Path, path: &Path) -> Result<()> {
    use std::os::unix::fs::symlink;

    symlink(target, path).with_context(|| format!("Failed to create Git archive symlink {}", path.display()))
}

#[cfg(windows)]
fn create_symlink(
    target: &Path,
    path: &Path,
    destination: &Path,
    symlink_targets: &HashMap<String, PathBuf>,
) -> Result<()> {
    use std::os::windows::fs::{symlink_dir, symlink_file};

    let create = if windows_symlink_target_is_directory(target, path, destination, symlink_targets)? {
        symlink_dir
    } else {
        symlink_file
    };
    create(target, path).with_context(|| {
        format!(
            "Failed to create Git archive symlink {}. Enable Windows Developer Mode to materialize committed symlinks without elevation",
            path.display()
        )
    })
}

#[cfg(windows)]
fn windows_symlink_target_is_directory(
    target: &Path,
    link: &Path,
    destination: &Path,
    symlink_targets: &HashMap<String, PathBuf>,
) -> Result<bool> {
    let resolved = resolve_windows_symlink_target(target, link, destination, symlink_targets)?;
    let metadata = fs::symlink_metadata(destination.join(&resolved)).with_context(|| {
        format!(
            "Cannot create Git archive symlink {} on Windows: target {} was not extracted; dangling symlinks are unsupported",
            link.display(),
            resolved.display()
        )
    })?;
    if metadata.file_type().is_dir() {
        Ok(true)
    } else if metadata.file_type().is_file() {
        Ok(false)
    } else {
        bail!(
            "Cannot create Git archive symlink {} on Windows: target {} is not a regular file or directory",
            link.display(),
            resolved.display()
        )
    }
}

#[cfg(windows)]
fn resolve_windows_symlink_target(
    target: &Path,
    link: &Path,
    destination: &Path,
    symlink_targets: &HashMap<String, PathBuf>,
) -> Result<PathBuf> {
    let link_relative = link.strip_prefix(destination).with_context(|| {
        format!(
            "Cannot create Git archive symlink {} on Windows: link is outside the export destination",
            link.display()
        )
    })?;
    let mut current_link = link_relative.to_path_buf();
    let mut current_target = target.to_path_buf();
    let mut visited = HashSet::new();

    loop {
        if !visited.insert(current_link.clone()) {
            bail!(
                "Cannot create Git archive symlink {} on Windows: target chain contains a cycle at {}",
                link.display(),
                current_link.display()
            );
        }
        let relative_target = current_target.strip_prefix(destination).unwrap_or(&current_target);
        let resolved = normalize_relative_path(current_link.parent().unwrap_or(Path::new("")), relative_target)
            .with_context(|| {
                format!(
                    "Cannot create Git archive symlink {} on Windows: target {} escapes the export destination",
                    link.display(),
                    current_target.display()
                )
            })?;
        let resolved_key = canonical_relative_path(&resolved)?;
        let Some(next_target) = symlink_targets.get(&resolved_key) else {
            return Ok(resolved);
        };
        current_link = resolved;
        current_target = next_target.clone();
    }
}

#[cfg(windows)]
fn normalize_relative_path(parent: &Path, target: &Path) -> Option<PathBuf> {
    if target.is_absolute() || target.to_str().is_none() || target.to_string_lossy().contains('\\') {
        return None;
    }
    let mut resolved = PathBuf::new();
    for component in parent.join(target).components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::ParentDir if !resolved.pop() => return None,
            Component::ParentDir | Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(resolved)
}

#[cfg(not(any(unix, windows)))]
fn create_symlink(_target: &Path, path: &Path) -> Result<()> {
    bail!("Git archive contains unsupported symlink {}", path.display())
}

pub fn clone_repository(url: &str, branch: &str, destination: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", branch, url])
        .arg(destination)
        .status()
        .with_context(|| format!("{} while cloning {url}", git_prerequisite()))?;
    if !status.success() {
        bail!("Failed to clone {url} release tag {branch}");
    }
    Ok(())
}

const fn git_prerequisite() -> &'static str {
    "Git is required; install Git for Windows and ensure git.exe is on PATH"
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
