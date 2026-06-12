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

/// Grants only the capabilities the PHP-FPM master needs to drop privileges and own the socket.
#[test]
fn laravel_php_fpm_service_grants_required_drop_capabilities() {
    let path = templates_root().join("laravel/infra/assets/php/site-php-fpm.service.j2");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("CapabilityBoundingSet=CAP_SETUID CAP_SETGID CAP_CHOWN"),
        "laravel PHP-FPM systemd service must allow setuid, setgid, and chown for master privilege drop\n{content}"
    );
    assert!(
        content.contains("AmbientCapabilities="),
        "laravel PHP-FPM systemd service must keep ambient capabilities empty despite bounding capabilities\n{content}"
    );
}

/// PHP-FPM config must include a [global] section so it is a valid FPM config, not just a pool snippet.
#[test]
fn laravel_php_fpm_config_includes_global_section() {
    let path = templates_root().join("laravel/infra/assets/php/php-fpm-pool.conf.j2");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("[global]"), "laravel PHP-FPM config must include a [global] section\n{content}");
    assert!(
        content.contains("error_log = {{ paths.runtime_socket_dir }}/php-fpm.log"),
        "laravel PHP-FPM config must log errors under the writable runtime socket directory\n{content}"
    );
    assert!(
        content.contains("daemonize = no"),
        "laravel PHP-FPM config must disable daemonizing for systemd\n{content}"
    );
}

/// Laravel nginx should prefer index.php but still fall back to index.html for placeholder releases.
#[test]
fn laravel_nginx_template_prefers_php_but_falls_back_to_html() {
    let path = templates_root().join("laravel/infra/assets/nginx/laravel-site-nginx.conf.j2");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("index index.php index.html;"),
        "laravel nginx config must prefer index.php but fall back to index.html\n{content}"
    );
}

/// PHP-FPM pool config uses the resolved paths.current instead of raw path construction.
#[test]
fn laravel_php_fpm_config_uses_resolved_current_path() {
    let path = templates_root().join("laravel/infra/assets/php/php-fpm-pool.conf.j2");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("chdir = {{ paths.current }}"),
        "laravel PHP-FPM config must use the resolved current path instead of manual construction\n{content}"
    );
    assert!(
        !content.contains("{{ project_root }}/current"),
        "laravel PHP-FPM config must not hardcode project_root/current\n{content}"
    );
}

/// Validates the rendered PHP-FPM configuration before starting the service.
#[test]
fn laravel_operations_validates_php_fpm_config_before_service_start() {
    let path = templates_root().join("laravel/infra/operations.py");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("--test --fpm-config"),
        "laravel operations must validate PHP-FPM config with --test before starting the service\n{content}"
    );
    let validate_idx = content.find("--test --fpm-config");
    let start_idx = content.find("Enable and start per-project PHP-FPM service");
    assert!(
        validate_idx.is_some() && start_idx.is_some() && validate_idx < start_idx,
        "laravel operations must validate PHP-FPM config before enabling and starting the service\n{content}"
    );
}

/// Uses an absolute nginx fastcgi params include because per-site configs run outside /etc/nginx.
#[test]
fn laravel_nginx_template_uses_absolute_fastcgi_params_include() {
    let path = templates_root().join("laravel/infra/assets/nginx/laravel-site-nginx.conf.j2");
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

/// Laravel nginx template uses the resolved path manifest so it stays in sync with systemd/AppArmor.
#[test]
fn laravel_nginx_template_uses_resolved_path_manifest() {
    let path = templates_root().join("laravel/infra/assets/nginx/laravel-site-nginx.conf.j2");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("pid {{ paths.runtime_nginx_pid }}"),
        "laravel nginx config must use the resolved pid path\n{content}"
    );
    assert!(
        content.contains("listen unix:{{ paths.runtime_nginx_socket }}"),
        "laravel nginx config must use the resolved listen socket path\n{content}"
    );
    assert!(
        content.contains("root {{ paths.current_web_root }}"),
        "laravel nginx config must use the resolved web root path\n{content}"
    );
    assert!(
        content.contains("{{ paths.runtime_socket_dir }}/"),
        "laravel nginx config must use the resolved socket directory for temp paths\n{content}"
    );
    assert!(
        !content.contains("/run/{{ project_name }}"),
        "laravel nginx config must not hardcode /run path instead of using the manifest\n{content}"
    );
    assert!(
        !content.contains("{{ project_root }}/current/{{ web_root }}"),
        "laravel nginx config must not manually construct current web root\n{content}"
    );
}

/// Laravel build script has an ERR trap that prints the failing command and line.
#[test]
fn laravel_build_script_has_err_trap_with_command_and_line() {
    let path = templates_root().join("laravel/deployment/02_run_build.sh");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("trap '") && content.contains("ERR"),
        "laravel build script must trap on ERR to report failing commands\n{content}"
    );
    assert!(
        content.contains("$LINENO") && content.contains("$BASH_COMMAND"),
        "laravel build script ERR trap must include the failing line number and command\n{content}"
    );
}

/// Laravel build script prints step labels before Composer, artisan, pnpm, migrations, and cache rebuilds
/// so that production failures are easy to localize in deploy output.
#[test]
fn laravel_build_script_prints_step_labels_for_phases() {
    let path = templates_root().join("laravel/deployment/02_run_build.sh");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    for label in &[
        "Installing Composer dependencies",
        "Entering Laravel maintenance mode",
        "Installing frontend dependencies",
        "Building frontend assets",
        "Running migrations",
        "Rebuilding Laravel caches",
    ] {
        assert!(content.contains(label), "laravel build script must print a step label for: {label}\n{content}");
    }
}

/// Laravel nginx config writes error/access logs under the runtime socket directory
/// so that the non-root service user can write them under the systemd sandbox and AppArmor profile.
#[test]
fn laravel_nginx_config_writes_logs_under_runtime_socket_dir() {
    let path = templates_root().join("laravel/infra/assets/nginx/laravel-site-nginx.conf.j2");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("error_log {{ paths.runtime_socket_dir }}/error.log"),
        "laravel nginx config must write error log under the runtime socket directory\n{content}"
    );
    assert!(
        content.contains("access_log {{ paths.runtime_socket_dir }}/access.log"),
        "laravel nginx config must write access log under the runtime socket directory\n{content}"
    );
    assert!(
        !content.contains("access_log stderr"),
        "laravel nginx config must not use relative stderr access log (non-root cannot write /usr/share/nginx)\n{content}"
    );
}
