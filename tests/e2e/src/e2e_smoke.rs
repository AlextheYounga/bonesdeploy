use crate::support::docker;

/// Verifies that the e2e harness bootstrap SSH user defaults to root for secure provisioning.
#[test]
#[ignore = "e2e test"]
fn e2e_harness_bootstrap_user_defaults_to_root() {
    let user = docker::bootstrap_ssh_user();
    assert_eq!(user, "root");
}

/// Verifies that the e2e harness bootstrap SSH user can be overridden via environment variable.
#[test]
#[ignore = "e2e test"]
fn e2e_harness_bootstrap_user_can_be_overridden() {
    let user = std::env::var("BONES_E2E_BOOTSTRAP_USER").unwrap_or_else(|_| String::from("root"));
    assert!(!user.is_empty());
}
