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

    let doctor = env.run(&["doctor", "--local"])?;
    assert!(
        doctor.status.success(),
        "doctor should pass on an initialized repo: {}",
        String::from_utf8_lossy(&doctor.stdout)
    );
    let stdout = String::from_utf8_lossy(&doctor.stdout);
    assert!(stdout.contains("All checks passed."), "expected a clean doctor report: {stdout}");
    Ok(())
}
