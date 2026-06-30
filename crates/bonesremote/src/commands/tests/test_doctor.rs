use std::collections::HashSet;

use super::{
    apparmor_kernel_enabled, apparmor_profile_filename, apparmor_unit_name_for_profile, apparmor_unit_wiring_issue,
};

#[test]
fn apparmor_kernel_enabled_accepts_yes() {
    assert!(apparmor_kernel_enabled("Y\n"));
}

#[test]
fn apparmor_kernel_enabled_rejects_no() {
    assert!(!apparmor_kernel_enabled("N\n"));
}

#[test]
fn apparmor_profile_filename_accepts_bonesdeploy_profile() {
    assert!(apparmor_profile_filename("bonesdeploy-demo-nginx"));
}

#[test]
fn apparmor_profile_filename_rejects_unrelated_file() {
    assert!(!apparmor_profile_filename("default"));
}

#[test]
fn apparmor_unit_name_for_profile_maps_project_unit() {
    assert_eq!(apparmor_unit_name_for_profile("bonesdeploy-demo-nginx"), Some("demo-nginx.service".to_string()));
}

#[test]
fn apparmor_unit_wiring_accepts_expected_unit_with_reordered_after_tokens() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(apparmor_unit_wiring_issue(
        "[Unit]\nAfter=apparmor.service network.target\nRequires=apparmor.service\n[Service]\nAppArmorProfile=bonesdeploy-demo-nginx\n",
        &installed_profiles,
    )
    .is_none());
}

#[test]
fn apparmor_unit_wiring_rejects_missing_profile_binding() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(
        apparmor_unit_wiring_issue(
            "[Unit]\nAfter=network.target apparmor.service\nRequires=apparmor.service\n[Service]\nType=simple\n",
            &installed_profiles,
        )
        .is_some()
    );
}

#[test]
fn apparmor_unit_wiring_rejects_unknown_profile_binding() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    let issue = apparmor_unit_wiring_issue(
        "[Unit]\nAfter=network.target apparmor.service\nRequires=apparmor.service\n[Service]\nAppArmorProfile=bonesdeploy-demo-ngnix\n",
        &installed_profiles,
    );
    assert!(issue.is_some_and(|msg| msg.contains("bonesdeploy-demo-ngnix")));
}
