use bonesdeploy_core::paths::DeploymentPaths;

#[test]
fn deployment_paths_derive_the_complete_site_layout() {
    let paths = DeploymentPaths::new("example", "/srv/sites/example", "public");

    assert_eq!(paths.project_root_parent, "/srv/sites");
    assert_eq!(paths.current_web_root, "/srv/sites/example/current/public");
    assert_eq!(paths.placeholder_index, "/srv/sites/example/releases/19700101_000000/public/index.html");
    assert_eq!(paths.nginx_site_available, "/etc/nginx/sites-available/example.conf");
    assert_eq!(paths.runtime_nginx_socket, "/run/example/nginx/nginx.sock");
    assert_eq!(paths.backup_passphrase_file, "/root/.config/bonesremote/sites/example/.borg_passphrase");
    assert_eq!(paths.systemd_service("worker"), "/etc/systemd/system/example-worker.service");
    assert_eq!(
        paths.systemd_service_requirement("worker"),
        "/etc/systemd/system/example.target.requires/example-worker.service"
    );
    assert_eq!(paths.apparmor_profile("worker"), "/etc/apparmor.d/bonesdeploy-example-worker");
    assert_eq!(paths.runtime_service_socket("redis"), "/run/example/redis/redis.sock");
}

#[test]
fn relative_project_roots_use_the_product_default_parent() {
    let paths = DeploymentPaths::new("example", "example", "dist");

    assert_eq!(paths.project_root_parent, "/srv/sites");
}
