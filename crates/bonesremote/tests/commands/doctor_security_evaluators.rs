use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bonesremote::commands::doctor::security::evaluators::{evaluate_active_release, evaluate_privileged_path};
use bonesremote::commands::doctor::security::types::{
    Account, CurrentState, FileKind, FileNode, PathTree, ReleaseEvidence, Status,
};

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

#[test]
fn writable_child_of_protected_directory_fails() {
    let tree = PathTree {
        requested: PathBuf::from("/etc/systemd/system"),
        nodes: vec![
            node("/", FileKind::Directory, 0, 0o755),
            node("/etc", FileKind::Directory, 0, 0o755),
            node("/etc/systemd", FileKind::Directory, 0, 0o755),
            node("/etc/systemd/system", FileKind::Directory, 0, 0o755),
            node("/etc/systemd/system/atlas.service", FileKind::File, 1001, 0o600),
        ]
        .into_iter()
        .map(|node| (node.path.clone(), node))
        .collect(),
    };
    let runtime = account();

    assert_eq!(evaluate_privileged_path(&tree, &[&runtime]).status, Status::Fail);
}

#[test]
fn broken_and_out_of_tree_current_targets_fail() {
    let runtime = account();
    let filesystem = PathTree { requested: PathBuf::from("/srv/sites/atlas/current"), nodes: BTreeMap::new() };
    let broken = ReleaseEvidence {
        site: "atlas".to_string(),
        releases_root: PathBuf::from("/srv/sites/atlas/releases"),
        current: CurrentState::Broken,
        filesystem: filesystem.clone(),
    };
    let outside = ReleaseEvidence {
        site: "atlas".to_string(),
        releases_root: PathBuf::from("/srv/sites/atlas/releases"),
        current: CurrentState::Active(PathBuf::from("/tmp/attacker-release")),
        filesystem,
    };

    assert_eq!(evaluate_active_release(&broken, &runtime).status, Status::Fail);
    assert_eq!(evaluate_active_release(&outside, &runtime).status, Status::Fail);
}
