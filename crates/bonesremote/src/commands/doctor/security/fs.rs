use std::path::Path;

use super::types::{Account, FileKind, FileNode, PathTree};

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Authority {
    Granted,
    Denied,
    Unverified(String),
}

pub(super) fn account_can_modify(path: &Path, account: &Account, tree: &PathTree) -> Authority {
    let Some(node) = tree.node(path) else {
        return Authority::Unverified(format!("metadata was not collected for {}", path.display()));
    };
    match can_traverse_to(path, account, tree) {
        Authority::Granted => {}
        decision => return decision,
    }
    if node.kind == FileKind::Symlink {
        return Authority::Denied;
    }
    if node.has_acl {
        return Authority::Unverified(format!("{} has a POSIX ACL that has not been evaluated", path.display()));
    }
    if node.uid == account.uid {
        return Authority::Granted;
    }
    let permission = permission_class(node, account);
    let writable = permission & 0o2 != 0;
    let searchable = node.kind != FileKind::Directory || permission & 0o1 != 0;
    if writable && searchable { Authority::Granted } else { Authority::Denied }
}

fn can_traverse_to(path: &Path, account: &Account, tree: &PathTree) -> Authority {
    let mut ancestors: Vec<_> = path.ancestors().skip(1).collect();
    ancestors.reverse();
    for ancestor in ancestors {
        let Some(node) = tree.node(ancestor) else {
            return Authority::Unverified(format!("ancestor metadata was not collected for {}", ancestor.display()));
        };
        if node.kind != FileKind::Directory {
            return Authority::Denied;
        }
        if node.has_acl {
            return Authority::Unverified(format!(
                "{} has a POSIX ACL that has not been evaluated",
                ancestor.display()
            ));
        }
        if node.uid == account.uid {
            continue;
        }
        if permission_class(node, account) & 0o1 == 0 {
            return Authority::Denied;
        }
    }
    Authority::Granted
}

fn permission_class(node: &FileNode, account: &Account) -> u32 {
    if node.uid == account.uid {
        (node.mode >> 6) & 0o7
    } else if account.groups.contains(&node.gid) {
        (node.mode >> 3) & 0o7
    } else {
        node.mode & 0o7
    }
}

pub(super) fn has_login_shell(shell: &str) -> bool {
    !matches!(shell, "/usr/sbin/nologin" | "/sbin/nologin" | "/bin/false")
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use super::{Authority, account_can_modify};
    use crate::commands::doctor::security::types::{Account, FileKind, FileNode, PathTree};

    fn account() -> Account {
        Account {
            name: "atlas".to_string(),
            uid: 1001,
            gid: 1001,
            shell: "/usr/sbin/nologin".to_string(),
            groups: BTreeSet::from([1001]),
        }
    }

    fn node(path: &str, kind: FileKind, uid: u32, mode: u32) -> FileNode {
        FileNode { path: PathBuf::from(path), kind, uid, gid: uid, mode, has_acl: false }
    }

    fn tree(mut nodes: Vec<FileNode>) -> PathTree {
        nodes.extend([node("/", FileKind::Directory, 0, 0o755), node("/srv", FileKind::Directory, 0, 0o755)]);
        PathTree {
            requested: PathBuf::from("/srv/sites"),
            nodes: nodes.into_iter().map(|node| (node.path.clone(), node)).collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn symlink_mode_bits_do_not_grant_replacement_authority() {
        let evidence = tree(vec![
            node("/srv/sites", FileKind::Directory, 0, 0o755),
            node("/srv/sites/current", FileKind::Symlink, 1001, 0o777),
        ]);

        assert_eq!(account_can_modify(Path::new("/srv/sites/current"), &account(), &evidence), Authority::Denied);
    }

    #[test]
    fn writable_file_behind_unsearchable_directory_is_not_effectively_writable() {
        let evidence = tree(vec![
            node("/srv/sites", FileKind::Directory, 0, 0o700),
            node("/srv/sites/unit.service", FileKind::File, 1001, 0o600),
        ]);

        assert_eq!(account_can_modify(Path::new("/srv/sites/unit.service"), &account(), &evidence), Authority::Denied);
    }

    #[test]
    fn acl_evidence_is_unverified_instead_of_passing() {
        let mut protected = node("/srv/sites", FileKind::Directory, 0, 0o755);
        protected.has_acl = true;
        let evidence = tree(vec![protected]);

        assert!(matches!(account_can_modify(Path::new("/srv/sites"), &account(), &evidence), Authority::Unverified(_)));
    }
}
