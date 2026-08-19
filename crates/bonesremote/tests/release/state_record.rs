use anyhow::Result;

use bonesremote::release::state::record::{DeploymentPhase, DeploymentRecord, ProcessIdentity};

#[test]
fn phases_after_commit_are_serialization_idle() {
    assert!(!DeploymentPhase::Created.is_committed());
    assert!(!DeploymentPhase::Sealed.is_committed());
    assert!(DeploymentPhase::Activated.is_committed());
    assert!(DeploymentPhase::Verified.is_committed());
    assert!(DeploymentPhase::CleanupPending.is_committed());
}

#[test]
fn cancellation_is_refused_after_runtime_mutation() {
    assert!(!DeploymentPhase::Built.may_have_mutated_runtime());
    assert!(DeploymentPhase::Promoted.may_have_mutated_runtime());
    assert!(DeploymentPhase::Prepared.may_have_mutated_runtime());
    assert!(DeploymentPhase::Verified.may_have_mutated_runtime());
}

#[test]
fn record_round_trips_through_json() -> Result<()> {
    let record = DeploymentRecord::new(
        String::from("20260804_190321-46a0b75c-a7f2"),
        String::from("46a0b75c"),
        DeploymentPhase::Verified,
        ProcessIdentity::new(1234, 42, String::from("2026-08-04T19:03:21Z")),
    );
    let json = serde_json::to_string(&record)?;
    let decoded: DeploymentRecord = serde_json::from_str(&json)?;
    assert_eq!(decoded.release(), record.release());
    assert_eq!(decoded.phase(), &DeploymentPhase::Verified);
    Ok(())
}
