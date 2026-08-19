use std::path::Path;

use super::types::{Account, FileKind, FileNode, PathTree};

#[derive(Debug, Eq, PartialEq)]
pub enum Authority {
    Granted,
    Denied,
    Unverified(String),
}

pub fn account_can_modify(path: &Path, account: &Account, tree: &PathTree) -> Authority {
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

pub fn has_login_shell(shell: &str) -> bool {
    !matches!(shell, "/usr/sbin/nologin" | "/sbin/nologin" | "/bin/false")
}
