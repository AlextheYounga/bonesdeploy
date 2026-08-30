mod common;

use anyhow::Result;

#[test]
fn exhaustive_doctor_requires_a_site() -> Result<()> {
    let output = common::run(&["doctor", "--exhaustive"])?;
    assert_eq!(output.status.code(), Some(2), "clap usage errors should exit with code 2");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--site"), "required-site error should mention --site: {stderr}");
    Ok(())
}

#[test]
fn exhaustive_doctor_accepts_a_site() -> Result<()> {
    let output = common::run(&["doctor", "--site", "atlas", "--exhaustive"])?;
    assert_ne!(output.status.code(), Some(2), "accepted arguments must not be a usage error");
    Ok(())
}

#[test]
fn config_sync_requires_config_stdin() -> Result<()> {
    let output = common::run(&["config", "sync", "--site", "atlas"])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--config-stdin"));
    Ok(())
}

#[test]
fn deploy_accepts_a_site_without_a_config_descriptor() -> Result<()> {
    let output = common::run(&["deploy", "--site", "atlas"])?;
    assert_ne!(output.status.code(), Some(2), "deploy must not require config stdin");
    Ok(())
}

#[test]
fn deploy_rejects_the_removed_config_stdin_option() -> Result<()> {
    let output = common::run(&["deploy", "--site", "atlas", "--config-stdin"])?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}
