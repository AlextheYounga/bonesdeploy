use std::{env, fs, process, process::Command};

use bonesremote::commands::doctor::site::check_branch_ref;

#[test]
fn empty_bare_repo_is_pending_before_first_push() {
    let root = env::temp_dir().join(format!("bonesremote-doctor-empty-repo-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    let output = Command::new("git").args(["init", "--bare", root.to_str().unwrap_or_default()]).output();
    assert!(output.is_ok_and(|output| output.status.success()));

    let mut issues = Vec::new();
    let mut pending = Vec::new();
    check_branch_ref(root.to_str().unwrap_or_default(), "master", &mut issues, &mut pending);

    let _ = fs::remove_dir_all(root);
    assert!(issues.is_empty());
    assert_eq!(pending.len(), 1);
}
