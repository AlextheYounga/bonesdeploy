mod common;

use anyhow::Result;

#[test]
fn doctor_accepts_verbose_flag() -> Result<()> {
    let env = common::TestEnv::new()?;
    let output = env.run(&["doctor", "--local", "--verbose"])?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checking deployment"), "doctor should run with --verbose: {stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Doctor found 1 issue."), "missing config should fail doctor: {stderr}");
    Ok(())
}
