use bonesremote::commands::release::kill::wait_for_process_exit;
use bonesremote::release::state::{DeploymentPhase, DeploymentRecord, ProcessIdentity};
use std::time::Duration;

#[test]
fn wait_returns_when_process_is_already_gone() {
    let deployment = DeploymentRecord::new(
        String::from("20260715_225306"),
        String::from("46a0b75c"),
        DeploymentPhase::Created,
        ProcessIdentity::new(u32::MAX, 0, String::new()),
    );
    assert!(wait_for_process_exit(&deployment, Duration::from_millis(1)));
}
