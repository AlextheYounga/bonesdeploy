use std::collections::HashSet;

use super::{
    AppArmorUnitWiringStatus, algif_aead_is_loaded, apparmor_kernel_enabled, apparmor_profile_binding,
    apparmor_profile_filename, apparmor_unit_name_for_profile, apparmor_unit_wiring_status,
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

/// Detects `algif_aead` in loaded kernel module list.
#[test]
fn algif_aead_is_loaded_detects_module() {
    assert!(algif_aead_is_loaded("algif_aead 20480 0\next4 1000000 3\n"));
}

/// Rejects module list without `algif_aead`.
#[test]
fn algif_aead_is_loaded_rejects_absent_module() {
    assert!(!algif_aead_is_loaded("ext4 1000000 3\n"));
}

/// Rejects module names that only contain `algif_aead` as a prefix.
#[test]
fn algif_aead_is_loaded_rejects_prefix_match() {
    assert!(!algif_aead_is_loaded("algif_aead_other 20480 0\n"));
}
