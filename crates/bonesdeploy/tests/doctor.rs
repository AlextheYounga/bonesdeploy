mod common;

use anyhow::Result;
use common::TestEnv;

const INIT_ARGS: &[&str] =
    &["init", "--non-interactive", "--project-name", "atlas", "--host", "deploy.example.com", "--branch", "master"];

#[test]
fn deployment_script_check_accepts_nested_build_and_prepare_layout() -> Result<()> {
    let env = TestEnv::new()?;
    let init = env.run(INIT_ARGS)?;
    assert!(init.status.success(), "init failed: {}", String::from_utf8_lossy(&init.stderr));
    common::commit_initial(env.repo())?;

    let doctor = env.run(&["site", "doctor", "--local"])?;
    assert!(
        doctor.status.success(),
        "doctor should pass on an initialized repo: {}",
        String::from_utf8_lossy(&doctor.stdout)
    );
    let stdout = String::from_utf8_lossy(&doctor.stdout);
    assert!(stdout.contains("All checks passed."), "expected a clean doctor report: {stdout}");
    Ok(())
}

#[test]
fn root_doctor_runs_site_diagnostics_after_a_server_failure() -> Result<()> {
    let env = TestEnv::new()?;
    let output = env.run(&["doctor"])?;

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checking deployment"), "site doctor did not run: {stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Server doctor failed"), "missing server failure: {stderr}");
    assert!(stderr.contains("Site doctor failed"), "missing site failure: {stderr}");
    Ok(())
}

#[test]
fn site_setup_stops_at_missing_server_baseline() -> Result<()> {
    let env = TestEnv::new()?;
    let init =
        env.run(&["init", "--non-interactive", "--project-name", "atlas", "--host", "127.0.0.1", "--port", "1"])?;
    assert!(init.status.success(), "init failed: {}", String::from_utf8_lossy(&init.stderr));

    let output = env.run(&["site", "setup", "--yes"])?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Server baseline is not ready."), "unexpected stderr: {stderr}");
    assert!(stderr.contains("Next: bonesdeploy server setup --yes"), "unexpected stderr: {stderr}");
    Ok(())
}
