use std::fs;
use std::path::Path;

/// Verifies the update deploy uses cargo install --git like the setup deploy.
#[test]
fn update_deploy_uses_cargo_install_from_git() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands/update_release.rs");
    let content = fs::read_to_string(&src);
    assert!(content.is_ok(), "failed to read {}", src.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("cargo install") && content.contains("--git"),
        "update must use cargo install --git for bonesremote\n{content}"
    );
    assert!(
        content.contains("write_update_deploy_file"),
        "update must generate a pyinfra deploy file for remote update\n{content}"
    );
    assert!(
        !content.contains("bonesremote_staging_path"),
        "update must not use release-style staged binary uploads\n{content}"
    );
}

/// Ensures runtime deploy maintains apparmor_profile_path derived from profile name pattern.
#[test]
fn runtime_deploy_derives_profile_path_from_profile_name() {
    let deploy = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/infra/runtime.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("bonesdeploy-{DEPLOY_DATA.project_name}-nginx"),
        "runtime deploy must derive profile name from project name\n{content}"
    );
    assert!(
        content.contains("apparmor_parser -r"),
        "runtime deploy must load the per-project apparmor profile\n{content}"
    );
    assert!(content.contains("aa-enforce"), "runtime deploy must set project profile to enforce mode\n{content}");
}
