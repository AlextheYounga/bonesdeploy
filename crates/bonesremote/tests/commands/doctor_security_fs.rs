use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bonesremote::commands::doctor::security::fs::{Authority, account_can_modify};
use bonesremote::commands::doctor::security::types::{Account, FileKind, FileNode, PathTree};

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
