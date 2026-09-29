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
fn config_sync_rejects_the_removed_config_stdin_option() -> Result<()> {
    let output = common::run(&["config", "sync", "--site", "atlas", "--config-stdin"])?;
    assert_eq!(output.status.code(), Some(2), "clap usage errors should exit with code 2");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--config-stdin"), "unknown-argument error should mention --config-stdin: {stderr}");
    Ok(())
}

#[test]
fn config_sync_accepts_a_site_argument() -> Result<()> {
    let output = common::run(&["config", "sync", "--site", "atlas"])?;
    assert_ne!(
        output.status.code(),
        Some(2),
        "accepted arguments must not be a usage error; sync always reads its descriptor from stdin"
    );
    Ok(())
}

#[test]
fn deploy_accepts_the_artifact_only_form() -> Result<()> {
    let output = common::run(&["deploy", "--site", "atlas"])?;
    assert_ne!(output.status.code(), Some(2), "artifact deployment must be accepted by clap");
    Ok(())
}

#[test]
fn deploy_rejects_the_removed_config_stdin_option() -> Result<()> {
    let output = common::run(&["deploy", "--site", "atlas", "--config-stdin"])?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}

#[test]
fn deploy_rejects_the_removed_artifact_stdin_option() -> Result<()> {
    let output = common::run(&["deploy", "--site", "atlas", "--artifact-stdin"])?;
    assert_eq!(output.status.code(), Some(2), "artifact stdin is implicit for the only deploy route");
    Ok(())
}

#[test]
fn deploy_rejects_the_removed_revision_option() -> Result<()> {
    let output = common::run(&["deploy", "--site", "atlas", "--revision", "main"])?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}

#[test]
fn decommission_protocol_commands_accept_a_site_argument() -> Result<()> {
    for command in ["begin", "complete", "verify", "reactivate"] {
        let output = common::run(&["decommission", command, "--site", "atlas"])?;
        assert_ne!(output.status.code(), Some(2), "{command} must accept --site");
    }
    Ok(())
}
