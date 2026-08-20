use bonesremote::commands::service::{configured_laravel_worker_service, target_name_for_registered_site};

#[test]
fn site_cannot_restart_another_projects_target() {
    assert!(target_name_for_registered_site("shop", "shop-admin").is_err());
}

#[test]
fn enabled_laravel_queue_workers_restart_after_cutover() {
    assert_eq!(configured_laravel_worker_service("laravel", true, "shop").as_deref(), Some("shop-worker.service"));
}

#[test]
fn other_projects_do_not_restart_a_laravel_queue_worker() {
    assert_eq!(configured_laravel_worker_service("next", true, "shop"), None);
}

#[test]
fn disabled_laravel_queue_workers_do_not_restart_after_cutover() {
    assert_eq!(configured_laravel_worker_service("laravel", false, "shop"), None);
}
