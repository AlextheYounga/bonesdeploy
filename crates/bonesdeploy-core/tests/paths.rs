//! Path derivation for the `bonesdeploy-core` library.

use bonesdeploy_core::paths;

#[test]
fn site_target_name_is_exactly_project_derived() {
    assert_eq!(paths::site_target_name("nexttest"), "nexttest.target");
    assert_ne!(paths::site_target_name("shop"), "shop-admin.target");
    assert_eq!(
        paths::bonesremote_site_config_path("nexttest").to_string_lossy(),
        "/root/.config/bonesremote/sites/nexttest/config.env"
    );
}
