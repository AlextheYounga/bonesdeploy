#[cfg(windows)]
use std::collections::HashMap;
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::ensure_git_repository;
use crate::test_support::with_env;

#[test]
fn missing_git_names_the_windows_prerequisite_and_executable() -> Result<()> {
    let result = with_env("PATH", Some("missing-git-path"), ensure_git_repository);
    let error = result.err().context("missing Git should fail")?;
    let message = format!("{error:#}");
    assert!(message.contains("Git is required"), "{message}");
    assert!(message.contains("git.exe"), "{message}");
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_symlink_target_classification_rejects_missing_targets() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("file"), "content")?;
    fs::create_dir(temp.path().join("directory"))?;

    let symlink_targets = HashMap::new();
    assert!(!super::windows_symlink_target_is_directory(
        Path::new("file"),
        &temp.path().join("file-link"),
        temp.path(),
        &symlink_targets
    )?);
    assert!(super::windows_symlink_target_is_directory(
        Path::new("directory"),
        &temp.path().join("directory-link"),
        temp.path(),
        &symlink_targets
    )?);
    let error = super::windows_symlink_target_is_directory(
        Path::new("missing"),
        &temp.path().join("link"),
        temp.path(),
        &symlink_targets,
    )
    .err()
    .context("missing Windows symlink target must fail")?;
    assert!(error.to_string().contains("dangling symlinks are unsupported"));
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_symlink_target_classification_resolves_forward_link_chains() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::create_dir(temp.path().join("dir"))?;
    fs::create_dir(temp.path().join("dir/target"))?;

    assert!(super::windows_symlink_target_is_directory(
        Path::new("second"),
        &temp.path().join("dir/first"),
        temp.path(),
        &HashMap::from([
            ("dir/first".to_owned(), PathBuf::from("second")),
            ("dir/second".to_owned(), PathBuf::from("target")),
        ])
    )?);
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_symlink_target_classification_rejects_cycles_and_escapes() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let cycles =
        HashMap::from([("first".to_owned(), PathBuf::from("second")), ("second".to_owned(), PathBuf::from("first"))]);
    let cycle_error = super::windows_symlink_target_is_directory(
        Path::new("second"),
        &temp.path().join("first"),
        temp.path(),
        &cycles,
    )
    .err()
    .context("symlink cycle must fail")?;
    assert!(cycle_error.to_string().contains("contains a cycle"));

    let escapes = HashMap::from([("first".to_owned(), PathBuf::from("../outside"))]);
    let escape_error = super::windows_symlink_target_is_directory(
        Path::new("first"),
        &temp.path().join("link"),
        temp.path(),
        &escapes,
    )
    .err()
    .context("symlink escape must fail")?;
    assert!(escape_error.to_string().contains("escapes the export destination"));
    Ok(())
}
