use std::{env, fs, os::unix::fs::MetadataExt, os::unix::fs::PermissionsExt, process};

use bonesremote::commands::doctor::baseline::check_etckeeper_executable;

#[test]
fn missing_etckeeper_executable_is_a_baseline_issue() {
    let path = env::temp_dir().join(format!("bonesremote-etckeeper-missing-{}", process::id()));
    let _ = fs::remove_file(&path);

    let mut issues = Vec::new();
    check_etckeeper_executable(&path, &mut issues);

    assert_eq!(issues.len(), 1);
    assert!(issues[0].starts_with("server baseline etckeeper executable"));
    assert!(issues[0].contains("is missing"), "unexpected issue: {}", issues[0]);
}

#[test]
fn non_regular_file_etckeeper_path_is_a_baseline_issue() {
    let root = env::temp_dir().join(format!("bonesremote-etckeeper-dir-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    assert!(fs::create_dir_all(&root).is_ok());
    let directory = root.join("etckeeper");
    assert!(fs::create_dir_all(&directory).is_ok());

    let mut issues = Vec::new();
    check_etckeeper_executable(&directory, &mut issues);

    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("must be a regular file"), "unexpected issue: {}", issues[0]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn insecure_etckeeper_executable_is_reported_and_secured_one_passes_for_root() {
    let root = env::temp_dir().join(format!("bonesremote-etckeeper-file-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    assert!(fs::create_dir_all(&root).is_ok());
    let executable = root.join("etckeeper");
    assert!(fs::write(&executable, b"#!/bin/sh\n").is_ok());
    assert!(fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).is_ok());

    let mut issues = Vec::new();
    check_etckeeper_executable(&executable, &mut issues);

    let metadata = fs::symlink_metadata(&executable);
    assert!(metadata.is_ok(), "could not read temp executable metadata: {:?}", metadata.as_ref().err());
    if metadata.as_ref().is_ok_and(|metadata| metadata.uid() == 0) {
        assert!(issues.is_empty(), "root-owned executable should pass: {issues:?}");
    } else {
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("must be owned by root:root"), "unexpected issue: {}", issues[0]);
    }

    let _ = fs::remove_dir_all(root);
}
