use std::fs;
use std::path::Path;

/// Ensures runtime deploy maintains apparmor_profile_path derived from profile name pattern.
#[test]
fn runtime_deploy_derives_profile_path_from_profile_name() {
    let deploy = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("infra/src/runtime.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("data['project_name']"),
        "runtime deploy must derive profile name from project name\n{content}"
    );
    assert!(
        content.contains("apparmor_parser -r"),
        "runtime deploy must load the per-project apparmor profile\n{content}"
    );
    assert!(content.contains("aa-enforce"), "runtime deploy must set project profile to enforce mode\n{content}");
}
