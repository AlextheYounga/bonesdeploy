use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::fs;
use std::io::{Error, ErrorKind};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::ptr::null_mut;

use shared::{config, paths};

use super::types::{
    Account, CurrentState, FileKind, FileNode, PathTree, PolicyDecision, ReleaseEvidence, Site, SudoEvidence,
};

pub(super) fn collect_accounts() -> Result<BTreeMap<String, Account>, String> {
    let passwd =
        fs::read_to_string(paths::ETC_PASSWD).map_err(|error| format!("cannot read {}: {error}", paths::ETC_PASSWD))?;
    let groups =
        fs::read_to_string(paths::ETC_GROUP).map_err(|error| format!("cannot read {}: {error}", paths::ETC_GROUP))?;
    let mut accounts = BTreeMap::new();
    for line in passwd.lines().filter(|line| !line.is_empty() && !line.starts_with('#')) {
        let fields: Vec<_> = line.split(':').collect();
        if fields.len() < 7 {
            return Err(format!("malformed passwd entry: {line}"));
        }
        let uid = fields[2].parse().map_err(|_| format!("invalid UID for {}", fields[0]))?;
        let gid = fields[3].parse().map_err(|_| format!("invalid GID for {}", fields[0]))?;
        accounts.insert(
            fields[0].to_string(),
            Account {
                name: fields[0].to_string(),
                uid,
                gid,
                shell: fields[6].to_string(),
                groups: BTreeSet::from([gid]),
            },
        );
    }
    for line in groups.lines().filter(|line| !line.is_empty() && !line.starts_with('#')) {
        let fields: Vec<_> = line.split(':').collect();
        if fields.len() < 4 {
            return Err(format!("malformed group entry: {line}"));
        }
        let gid = fields[2].parse().map_err(|_| format!("invalid group ID for {}", fields[0]))?;
        for member in fields[3].split(',').filter(|member| !member.is_empty()) {
            let account = accounts
                .get_mut(member)
                .ok_or_else(|| format!("group {} references unknown user {member}", fields[0]))?;
            account.groups.insert(gid);
        }
    }
    Ok(accounts)
}

pub(super) fn collect_sites(accounts: &BTreeMap<String, Account>) -> Result<Vec<Site>, String> {
    let root = paths::bonesremote_sites_root();
    let entries = fs::read_dir(&root).map_err(|error| format!("cannot read {}: {error}", root.display()))?;
    let mut sites = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot enumerate {}: {error}", root.display()))?;
        if !entry.file_type().map_err(|error| error.to_string())?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let cfg = config::load(&entry.path().join(paths::BONES_TOML))
            .map_err(|error| format!("cannot load site {name}: {error}"))?;
        if cfg.project_name != name {
            return Err(format!("site directory {name} contains configuration for {}", cfg.project_name));
        }
        let runtime_name = config::runtime_user_for(&name);
        let build_name = config::build_user_for(&name);
        let runtime =
            accounts.get(&runtime_name).cloned().ok_or_else(|| format!("runtime user {runtime_name} is absent"))?;
        let build = accounts.get(&build_name).cloned().ok_or_else(|| format!("build user {build_name} is absent"))?;
        sites.push(Site {
            name,
            project_root: PathBuf::from(&cfg.project_root),
            runtime: collect_identity_groups(runtime)?,
            build: collect_identity_groups(build)?,
        });
    }
    Ok(sites)
}

pub(super) fn collect_path_tree(path: &Path, follow_symlink_targets: bool) -> Result<PathTree, String> {
    let mut nodes = BTreeMap::new();
    let mut walked = BTreeSet::new();
    collect_parents(path, &mut nodes)?;
    walk_path(path, follow_symlink_targets, &mut nodes, &mut walked)?;
    Ok(PathTree { requested: path.to_path_buf(), nodes: nodes.into_values().collect() })
}

pub(super) fn collect_release(site: &Site) -> Result<ReleaseEvidence, String> {
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
        walk_path(target, false, &mut nodes, &mut walked)?;
    }

    Ok(ReleaseEvidence {
        site: site.name.clone(),
        releases_root,
        current,
        filesystem: PathTree { requested: current_path, nodes: nodes.into_values().collect() },
    })
}

pub(super) fn collect_identity_groups(mut account: Account) -> Result<Account, String> {
    let output = Command::new("id")
        .args(["-G", "--", &account.name])
        .output()
        .map_err(|error| format!("cannot resolve supplementary groups for {}: {error}", account.name))?;
    if !output.status.success() {
        return Err(format!(
            "cannot resolve supplementary groups for {}: {}",
            account.name,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let output = String::from_utf8(output.stdout)
        .map_err(|error| format!("group IDs for {} are not valid UTF-8: {error}", account.name))?;
    let groups = output
        .split_whitespace()
        .map(|group| group.parse().map_err(|_| format!("invalid group ID for {}: {group}", account.name)))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if groups.is_empty() || !groups.contains(&account.gid) {
        return Err(format!("resolved groups for {} do not contain primary GID {}", account.name, account.gid));
    }
    account.groups = groups;
    Ok(account)
}

pub(super) fn collect_sudo_policy(user: &str) -> SudoEvidence {
    let mut sudo = Command::new("sudo");
    sudo.args(["-n", "-u", user, "sudo", "-n", "-l"]);
    sudo.env("LC_ALL", "C");

    let decision = match sudo.output() {
        Ok(output) if output.status.success() => PolicyDecision::Allowed,
        Ok(output) => classify_sudo_denial(output.status.code(), &String::from_utf8_lossy(&output.stderr)),
        Err(error) => PolicyDecision::Unverified(format!("could not execute sudo policy check: {error}")),
    };
    SudoEvidence { user: user.to_string(), decision }
}

fn classify_sudo_denial(status: Option<i32>, stderr: &str) -> PolicyDecision {
    let normalized = stderr.to_ascii_lowercase();
    let inspection_failure = normalized.contains("password is required")
        || normalized.contains("parse error")
        || normalized.contains("syntax error")
        || normalized.contains("unable to initialize")
        || normalized.contains("no valid sudoers")
        || normalized.contains("fatal");
    if status == Some(1) && !inspection_failure {
        PolicyDecision::Denied
    } else {
        let detail = stderr.trim();
        PolicyDecision::Unverified(if detail.is_empty() {
            format!("sudo policy check exited with status {status:?} without a policy decision")
        } else {
            format!("sudo policy check failed: {detail}")
        })
    }
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

#[cfg(test)]
mod tests {
    use super::PolicyDecision;
    use super::classify_sudo_denial;

    #[test]
    fn sudo_policy_denial_is_distinct_from_collector_failure() {
        assert_eq!(
            classify_sudo_denial(Some(1), "Sorry, user atlas is not allowed to execute '/bin/sh' as root"),
            PolicyDecision::Denied
        );
        assert_eq!(classify_sudo_denial(Some(1), ""), PolicyDecision::Denied);
        assert!(matches!(classify_sudo_denial(Some(1), "sudo: a password is required"), PolicyDecision::Unverified(_)));
        assert!(matches!(classify_sudo_denial(Some(2), "sudo: parse error"), PolicyDecision::Unverified(_)));
    }
}
