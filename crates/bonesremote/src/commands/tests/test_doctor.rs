use std::collections::HashSet;

use super::{
    AppArmorUnitWiringStatus, apparmor_kernel_enabled, apparmor_profile_binding, apparmor_profile_filename,
    apparmor_unit_name_for_profile, apparmor_unit_wiring_status, deploy_user_sudo_check_command,
};

/// Accepts a yes value as indicating `AppArmor` is enabled in the kernel.
#[test]
fn apparmor_kernel_enabled_accepts_yes() {
    assert!(apparmor_kernel_enabled("Y\n"));
}

/// Rejects a no value as indicating `AppArmor` is not enabled in the kernel.
#[test]
fn apparmor_kernel_enabled_rejects_no() {
    assert!(!apparmor_kernel_enabled("N\n"));
}

/// Accepts a valid bonesdeploy `AppArmor` profile filename.
#[test]
fn apparmor_profile_filename_accepts_bonesdeploy_profile() {
    assert!(apparmor_profile_filename("bonesdeploy-demo-nginx"));
}

/// Rejects a filename that does not match the bonesdeploy profile naming convention.
#[test]
fn apparmor_profile_filename_rejects_unrelated_file() {
    assert!(!apparmor_profile_filename("default"));
}

/// Maps a bonesdeploy `AppArmor` profile name to its corresponding systemd unit name.
#[test]
fn apparmor_unit_name_for_profile_maps_project_unit() {
    assert_eq!(apparmor_unit_name_for_profile("bonesdeploy-demo-nginx"), Some("demo-nginx.service".to_string()));
}

/// Accepts a systemd unit with correctly wired `AppArmor` dependency and profile.
#[test]
fn apparmor_unit_wiring_accepts_expected_unit_with_reordered_after_tokens() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(matches!(
        apparmor_unit_wiring_status(
            "[Unit]\nAfter=apparmor.service network.target\nRequires=apparmor.service\n[Service]\nAppArmorProfile=bonesdeploy-demo-nginx\n",
            &installed_profiles,
        ),
        AppArmorUnitWiringStatus::Ok
    ));
}

/// Rejects a systemd unit that is missing the `AppArmor` profile binding.
#[test]
fn apparmor_unit_wiring_rejects_missing_profile_binding() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(matches!(
        apparmor_unit_wiring_status(
            "[Unit]\nAfter=network.target apparmor.service\nRequires=apparmor.service\n[Service]\nType=simple\n",
            &installed_profiles,
        ),
        AppArmorUnitWiringStatus::MissingProfile
    ));
}

/// Rejects a systemd unit that binds an unknown `AppArmor` profile.
#[test]
fn apparmor_unit_wiring_rejects_unknown_profile_binding() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(matches!(
        apparmor_unit_wiring_status(
            "[Unit]\nAfter=network.target apparmor.service\nRequires=apparmor.service\n[Service]\nAppArmorProfile=bonesdeploy-demo-ngnix\n",
            &installed_profiles,
        ),
        AppArmorUnitWiringStatus::UnknownProfile(profile_name) if profile_name == "bonesdeploy-demo-ngnix"
    ));
}

/// Reads the first `AppArmor` profile assignment from a systemd unit file.
#[test]
fn apparmor_profile_binding_reads_first_profile_assignment() {
    assert_eq!(
        apparmor_profile_binding("[Service]\nAppArmorProfile=bonesdeploy-demo-nginx\n"),
        Some("bonesdeploy-demo-nginx")
    );
}

#[test]
fn sudoers_check_runs_as_deploy_user() {
    let command = deploy_user_sudo_check_command(["bonesremote", "hook", "post-receive", "--site", "demo"]);
    let args = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();

    assert_eq!(
        args,
        vec!["-n", "-u", "git", "sudo", "-n", "-l", "bonesremote", "hook", "post-receive", "--site", "demo"]
    );
}
