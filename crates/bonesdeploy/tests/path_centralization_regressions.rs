use std::fs;
use std::path::Path;

#[test]
fn update_bonesremote_playbook_validates_all_required_path_vars() {
    let playbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("updates/playbooks/update-bonesremote.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    for required_var in [
        "bonesremote_install_root is defined and bonesremote_install_root | length > 0",
        "bonesremote_stable_link is defined and bonesremote_stable_link | length > 0",
        "bonesremote_global_link is defined and bonesremote_global_link | length > 0",
        "bonesremote_managed_projects_root is defined and bonesremote_managed_projects_root | length > 0",
        "bonesremote_binary_name is defined and bonesremote_binary_name | length > 0",
        "bonesremote_swap_link_prefix is defined and bonesremote_swap_link_prefix | length > 0",
    ] {
        assert!(
            content.contains(required_var),
            "update-bonesremote playbook must fail fast when {required_var} is missing\n{content}"
        );
    }
}

#[test]
fn apparmor_role_keeps_profile_path_derived_from_profile_name_override() {
    let defaults =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/defaults/main.yml");
    let defaults_content = fs::read_to_string(&defaults);
    assert!(defaults_content.is_ok(), "failed to read {}", defaults.display());
    let defaults_content = defaults_content.unwrap_or_default();

    assert!(
        defaults_content.contains("apparmor_profile_path: \"/etc/apparmor.d/{{ apparmor_profile_name }}\""),
        "apparmor defaults must keep destination path derived from apparmor_profile_name\n{defaults_content}"
    );

    let tasks = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
    let tasks_content = fs::read_to_string(&tasks);
    assert!(tasks_content.is_ok(), "failed to read {}", tasks.display());
    let tasks_content = tasks_content.unwrap_or_default();

    assert!(
        tasks_content.contains("dest: \"{{ apparmor_profile_path }}\"")
            && tasks_content.contains("- \"{{ apparmor_profile_path }}\""),
        "apparmor tasks must consistently use apparmor_profile_path so profile-name overrides stay coherent\n{tasks_content}"
    );
}
