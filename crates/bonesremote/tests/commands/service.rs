use bonesremote::commands::service::target_name_for_registered_site;

#[test]
fn site_cannot_restart_another_projects_target() {
    assert!(target_name_for_registered_site("shop", "shop-admin").is_err());
}
