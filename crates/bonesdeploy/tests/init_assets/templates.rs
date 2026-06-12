use std::fs;

use std::path::Path;

fn templates_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("embeds/runtimes")
}

/// Keeps the Laravel runtime operations using host.data instead of a bare data global.
#[test]
fn laravel_runtime_operations_uses_host_data() {
    let path = templates_root().join("laravel/infra/operations.py");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("host.data"), "laravel runtime operations should use host.data\n{content}");
}

/// Runs the PHP-FPM master process without forcing the systemd service itself to the app user.
#[test]
fn laravel_php_fpm_service_template_leaves_privilege_dropping_to_the_pool() {
    let path = templates_root().join("laravel/infra/assets/php/site-php-fpm.service.j2");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("User={{ service_user }}"),
        "laravel PHP-FPM systemd service should not force the master process to the app user\n{content}"
    );
    assert!(
        !content.contains("Group={{ group }}"),
        "laravel PHP-FPM systemd service should not force the master process to the app group\n{content}"
    );
    assert!(
        content.contains("ExecStart=/usr/sbin/php-fpm{{ laravel_php_version_resolved }} --nodaemonize --fpm-config {{ laravel_php_fpm_pool_config_path }}"),
        "laravel PHP-FPM systemd service must still start the versioned FPM binary with the pool config\n{content}"
    );
}

/// Uses an absolute nginx fastcgi params include because per-site configs run outside /etc/nginx.
#[test]
fn laravel_nginx_template_uses_absolute_fastcgi_params_include() {
    let path = templates_root().join("laravel/infra/assets/nginx/site-nginx.conf.j2");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("include /etc/nginx/fastcgi_params;"),
        "laravel nginx site config must include the system fastcgi params file by absolute path\n{content}"
    );
    assert!(
        !content.contains("include fastcgi_params;"),
        "laravel nginx site config must not use a relative fastcgi_params include\n{content}"
    );
}
