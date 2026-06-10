use std::fs;
use std::path::Path;

/// Verifies the update-bonesremote playbook force-installs from git instead of uninstalling first or staging binaries.
#[test]
fn update_bonesremote_playbook_installs_from_git_like_init() {
    let playbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("updates/playbooks/update-bonesremote.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    for required_var in [
        "bonesremote_repo_url is defined and bonesremote_repo_url | length > 0",
        "bonesremote_install_root is defined and bonesremote_install_root | length > 0",
        "bonesremote_binary_path is defined and bonesremote_binary_path | length > 0",
        "bonesremote_managed_projects_root is defined and bonesremote_managed_projects_root | length > 0",
        "cargo_binary_path is defined and cargo_binary_path | length > 0",
    ] {
        assert!(
            content.contains(required_var),
            "update-bonesremote playbook must fail fast when {required_var} is missing\n{content}"
        );
    }

    assert!(
        !content.contains("- uninstall")
            && content.contains("- install")
            && content.contains("- --git")
            && content.contains("- --force"),
        "update-bonesremote playbook must use cargo install --git --force without a prior uninstall\n{content}"
    );

    assert!(
        !content.contains("bonesremote_staging_path") && !content.contains("remote_src: true"),
        "update-bonesremote playbook must not use release-style staged binary uploads\n{content}"
    );
}

/// Ensures `AppArmor` role defaults and tasks use `apparmor_profile_path` consistently.
#[test]
fn apparmor_role_keeps_profile_path_derived_from_profile_name_override() {
    let defaults =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/.lib/remote/roles/apparmor/defaults/main.yml");
    let defaults_content = fs::read_to_string(&defaults);
    assert!(defaults_content.is_ok(), "failed to read {}", defaults.display());
    let defaults_content = defaults_content.unwrap_or_default();

    assert!(
        defaults_content.contains("apparmor_profile_path: \"/etc/apparmor.d/{{ apparmor_profile_name }}\""),
        "apparmor defaults must keep destination path derived from apparmor_profile_name\n{defaults_content}"
    );

    let tasks =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/.lib/remote/roles/apparmor/tasks/main.yml");
    let tasks_content = fs::read_to_string(&tasks);
    assert!(tasks_content.is_ok(), "failed to read {}", tasks.display());
    let tasks_content = tasks_content.unwrap_or_default();

    assert!(
        tasks_content.contains("dest: \"{{ apparmor_profile_path }}\"")
            && tasks_content.contains("- \"{{ apparmor_profile_path }}\""),
        "apparmor tasks must consistently use apparmor_profile_path so profile-name overrides stay coherent\n{tasks_content}"
    );
}
