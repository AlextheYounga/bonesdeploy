//! Path derivation for the `bonesdeploy-core` library.

use bonesdeploy_core::paths;
use std::path::PathBuf;

#[test]
fn site_target_name_is_exactly_project_derived() {
    assert_eq!(paths::site_target_name("nexttest"), "nexttest.target");
    assert_ne!(paths::site_target_name("shop"), "shop-admin.target");
}

#[test]
fn remote_paths_remain_linux_paths() {
    assert_eq!(paths::bonesremote_config_root(), PathBuf::from("/root/.config/bonesremote"));
    assert_eq!(paths::bonesremote_sites_root(), PathBuf::from("/root/.config/bonesremote/sites"));
}

#[cfg(windows)]
#[test]
fn windows_roots_use_application_data_locations() {
    let app_data = std::env::var_os("APPDATA").expect("Windows provides APPDATA");
    let local_app_data = std::env::var_os("LOCALAPPDATA").expect("Windows provides LOCALAPPDATA");

    assert_eq!(paths::bones_config_root(), PathBuf::from(app_data).join("bonesdeploy"));
    assert_eq!(paths::bones_data_root(), PathBuf::from(&local_app_data).join("bonesdeploy"));
    assert_eq!(paths::bones_cache_root(), paths::bones_data_root());
    assert_eq!(paths::bones_state_root(), paths::bones_data_root());
}
