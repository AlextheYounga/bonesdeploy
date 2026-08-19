use bonesremote::commands::doctor::security::collection::sudo::{
    classify_sudo_denial, command_output, sudo_listing_denies,
};
use bonesremote::commands::doctor::security::types::PolicyDecision;

#[test]
fn sudo_policy_denial_is_distinct_from_collector_failure() {
    assert_eq!(
        classify_sudo_denial(Some(1), b"", b"Sorry, user atlas is not allowed to execute '/bin/sh' as root"),
        PolicyDecision::Denied
    );
    assert_eq!(classify_sudo_denial(Some(1), b"", b""), PolicyDecision::Denied);
    assert!(matches!(
        classify_sudo_denial(Some(1), b"", b"sudo: a password is required"),
        PolicyDecision::Unverified(_)
    ));
    assert!(matches!(classify_sudo_denial(Some(2), b"", b"sudo: parse error"), PolicyDecision::Unverified(_)));
}

#[test]
fn sudo_policy_output_keeps_stdout_and_stderr() {
    assert_eq!(command_output(b"allowed command", b"audit detail"), "stdout:\nallowed command\nstderr:\naudit detail");
}

#[test]
fn sudo_listing_denial_is_not_authority() {
    assert!(sudo_listing_denies(b"User e2evue is not allowed to run sudo on bones-e2e-487602-30de0aa3.\n", b""));
    assert!(!sudo_listing_denies(b"User e2evue may run the following commands on host:\n    (ALL) ALL\n", b""));
}
