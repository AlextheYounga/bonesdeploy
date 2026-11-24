mod common;

use anyhow::{Result, ensure};
use common::TestEnv;

#[test]
fn deletion_reports_resource_tree_and_services_before_connecting() -> Result<()> {
    let env = TestEnv::new()?;
    let init =
        env.run(&["init", "--non-interactive", "--project-name", "atlas", "--host", "127.0.0.1", "--port", "1"])?;
    ensure!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));

    let output = env.run(&["site", "delete", "--yes"])?;
    assert!(!output.status.success(), "unreachable host must stop deletion");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Remote site to delete: atlas"), "{stdout}");
    assert!(stdout.contains("Resources to remove\n/\n"), "{stdout}");
    assert!(stdout.contains("sites/"), "{stdout}");
    assert!(stdout.contains("atlas/"), "{stdout}");
    assert!(stdout.contains("Services to stop"), "{stdout}");
    assert!(stdout.contains("atlas.target"), "{stdout}");
    assert!(!stdout.contains("owner"), "{stdout}");
    Ok(())
}
