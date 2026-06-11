use std::fs;

use super::{TEMPLATES, project_root};

/// Uses the project name as the default service user instead of a hardcoded value.
#[test]
fn template_service_user_defaults_to_project_name_not_applications() {
    for template in TEMPLATES {
        let content = fs::read_to_string(project_root().join(template));
        assert!(content.is_ok(), "failed to read {template}");
        let content = content.unwrap_or_default();

        assert!(
            !content.contains("service_user: 'applications'"),
            "template {template} still hardcodes applications as the service user\n{content}"
        );
    }
}

/// Keeps the Laravel runtime operations using host.data instead of a bare data global.
#[test]
fn laravel_runtime_operations_uses_host_data() {
    let path = project_root().join("templates/laravel/runtime/operations.py");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("host.data"), "laravel runtime operations should use host.data\n{content}");
}

/// Runs the PHP-FPM master process without forcing the systemd service itself to the app user.
#[test]
fn laravel_php_fpm_service_template_leaves_privilege_dropping_to_the_pool() {
    let path = project_root().join("templates/laravel/runtime/templates/site-php-fpm.service.j2");
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
    let path = project_root().join("templates/laravel/runtime/nginx/site-nginx.conf.j2");
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

/// Does not install global npm packages in SPA template runtime operations.
#[test]
fn spa_template_runtime_operations_do_not_install_global_npm_packages() {
    for template in ["next", "sveltekit", "vue"] {
        let ops = project_root().join(format!("templates/{template}/runtime/operations.py"));
        let content = fs::read_to_string(&ops);
        assert!(content.is_ok(), "failed to read {}", ops.display());
        let content = content.unwrap_or_default();

        assert!(
            !content.contains("npm install -g"),
            "{template} runtime operations should not install globals during setup because the project Node version is resolved later\n{content}"
        );
    }
}
