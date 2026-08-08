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
