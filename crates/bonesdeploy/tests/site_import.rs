mod common;

use std::fs;
use std::os::unix::fs::symlink;

use anyhow::Result;

#[test]
fn missing_archive_fails_before_project_configuration_or_ssh() -> Result<()> {
    let env = common::TestEnv::new()?;

    let output = env.run(&["site", "import", "missing.zip", "--yes"])?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to open archive"), "unexpected error: {stderr}");
    assert!(!stderr.contains("project configuration"), "archive validation must happen first: {stderr}");
    Ok(())
}

#[test]
fn directory_and_symlink_archives_are_rejected_before_ssh() -> Result<()> {
    let env = common::TestEnv::new()?;
    fs::create_dir(env.repo().join("archive.zip"))?;

    let output = env.run(&["site", "import", "archive.zip", "--yes"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a regular file"));

    fs::remove_dir(env.repo().join("archive.zip"))?;
    fs::write(env.repo().join("actual.zip"), b"not read because project configuration is absent")?;
    symlink("actual.zip", env.repo().join("archive.zip"))?;

    let output = env.run(&["site", "import", "archive.zip", "--yes"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Failed to open archive"));
    Ok(())
}
