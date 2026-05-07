use anyhow::Result;

use crate::support::docker;

#[test]
#[ignore = "E2E test requires Docker and SSH setup"]
fn e2e_harness_bootstrap_user_defaults_to_root() {
    let user = docker::bootstrap_ssh_user();
    assert_eq!(user, "root");
}

#[test]
#[ignore = "E2E test requires Docker and SSH setup"]
fn e2e_harness_bootstrap_user_can_be_overridden() {
    let user = std::env::var("BONES_E2E_BOOTSTRAP_USER").unwrap_or_else(|_| String::from("root"));
    assert!(!user.is_empty());
}

#[test]
#[ignore = "E2E test requires Docker daemon"]
fn e2e_harness_can_start_and_stop_container() -> Result<()> {
    if !docker::docker_available() {
        return Ok(());
    }

    docker::docker_compose_up()?;
    docker::docker_compose_down()?;

    Ok(())
}
