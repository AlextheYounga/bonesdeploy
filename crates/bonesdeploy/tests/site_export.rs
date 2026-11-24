mod common;

use std::fs;

use anyhow::Result;
use common::TestEnv;

#[test]
fn export_refuses_an_existing_archive_without_modifying_it() -> Result<()> {
    let env = TestEnv::new()?;
    let init =
        env.run(&["init", "--non-interactive", "--project-name", "atlas", "--host", "127.0.0.1", "--port", "1"])?;
    assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    let destination = env.repo().join("existing.zip");
    fs::write(&destination, b"existing archive")?;

    let output = env.run(&["site", "export", "--output", "existing.zip"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Refusing to overwrite existing export"));
    assert_eq!(fs::read(destination)?, b"existing archive");
    Ok(())
}

#[test]
fn failed_export_removes_its_temporary_archive() -> Result<()> {
    let env = TestEnv::new()?;
    let init =
        env.run(&["init", "--non-interactive", "--project-name", "atlas", "--host", "127.0.0.1", "--port", "1"])?;
    assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    let destination = env.repo().join("exports");
    fs::create_dir(&destination)?;

    let output = env.run(&["site", "export", "--output", "exports/archive.zip"])?;
    assert!(!output.status.success());
    assert_eq!(fs::read_dir(destination)?.count(), 0, "failed transfer left an archive or staging file");
    Ok(())
}
