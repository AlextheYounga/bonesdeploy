"""J2 template file existence and content assertions."""

from . import helpers

N = helpers.SRC_DIR


def _read(name):
    return helpers.read(N / name)


# ---- Base AppArmor profile ----


def test_apparmor_profile_allows_resolved_web_root():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "{{ paths.current_web_root }}/** r,")
    helpers.assert_contains(c, "{{ paths.releases }}/*/{{ web_root }}/** r,")


def test_apparmor_profile_allows_site_nginx_conf():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "{{ paths.site_nginx_config }} r,")


def test_apparmor_profile_allows_repo_bones_toml():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "{{ paths.repo_bones_toml }} r,")


def test_apparmor_profile_does_not_deny_home_globally():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_not_contains(c, "deny /home/** r,")
    helpers.assert_not_contains(c, "deny /home/{{ deploy_user }}/** r,")


def test_apparmor_profile_limits_network_to_unix_stream():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "network unix stream,")
    helpers.assert_not_contains(c, "network inet stream,")
    helpers.assert_not_contains(c, "network inet6 stream,")


# ---- Base nginx service template ----


def test_nginx_service_sets_apparmor_profile():
    c = _read("assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "AppArmorProfile=")


def test_nginx_service_waits_for_apparmor():
    c = _read("assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "After=network.target apparmor.service")
    helpers.assert_contains(c, "Requires=apparmor.service")


# ---- Base nginx config ----


def test_site_nginx_config_logs_under_runtime_socket_dir():
    c = _read("assets/nginx/site-nginx.conf.j2")
    helpers.assert_contains(c, "error_log {{ paths.runtime_socket_dir }}/error.log")
    helpers.assert_contains(c, "access_log {{ paths.runtime_socket_dir }}/access.log")
    helpers.assert_not_contains(c, "access_log stderr")


# ---- Router nginx config ----


def test_router_config_uses_resolved_socket_path():
    c = _read("assets/nginx/router.conf.j2")
    helpers.assert_contains(c, "{{ paths.runtime_nginx_socket }}")
    helpers.assert_not_contains(c, "default_server")


# ---- Laravel PHP-FPM pool config ----


def test_laravel_php_fpm_config_has_global_section():
    c = _read("runtimes/laravel/assets/php/php-fpm-pool.conf.j2")
    helpers.assert_contains(c, "[global]")
    helpers.assert_contains(c, "error_log = /proc/self/fd/2")
    helpers.assert_contains(c, "daemonize = no")


def test_laravel_php_fpm_config_uses_resolved_current_path():
    c = _read("runtimes/laravel/assets/php/php-fpm-pool.conf.j2")
    helpers.assert_contains(c, "chdir = {{ paths.current }}")
    helpers.assert_not_contains(c, "{{ project_root }}/current")


# ---- Laravel PHP-FPM systemd service ----


def test_laravel_php_fpm_service_runs_as_runtime_user():
    c = _read("runtimes/laravel/assets/php/site-php-fpm.service.j2")
    helpers.assert_contains(c, "User={{ runtime_user }}")
    helpers.assert_contains(c, "Group={{ runtime_group }}")
    helpers.assert_contains(c, "SupplementaryGroups={{ release_group }}")
    helpers.assert_contains(
        c,
        "ExecStart=/usr/sbin/php-fpm{{ laravel_php_version_resolved }} "
        "--nodaemonize --fpm-config {{ laravel_php_fpm_pool_config_path }}",
    )
    helpers.assert_contains(c, "RuntimeDirectory={{ project_name }}")
    helpers.assert_contains(c, "RuntimeDirectoryMode=0750")
    helpers.assert_contains(c, "StandardOutput=journal")
    helpers.assert_contains(c, "StandardError=journal")


def test_laravel_php_fpm_service_grants_required_capabilities():
    c = _read("runtimes/laravel/assets/php/site-php-fpm.service.j2")
    helpers.assert_contains(c, "CapabilityBoundingSet=CAP_SETUID CAP_SETGID CAP_CHOWN")
    helpers.assert_contains(c, "AmbientCapabilities=")


# ---- Laravel PHP-FPM AppArmor profile ----


def test_laravel_php_fpm_apparmor_allows_site_conf_root():
    c = _read("runtimes/laravel/assets/php/site-php-fpm-profile.j2")
    helpers.assert_contains(c, "{{ paths.conf_root }}/ r,")
    helpers.assert_contains(c, "{{ paths.conf_root }}/** r,")


def test_laravel_php_fpm_apparmor_has_minimal_capabilities():
    c = _read("runtimes/laravel/assets/php/site-php-fpm-profile.j2")
    helpers.assert_contains(c, "capability chown,")
    helpers.assert_contains(c, "capability setgid,")
    helpers.assert_contains(c, "capability setuid,")
    helpers.assert_not_contains(c, "capability dac_override,")
    helpers.assert_not_contains(c, "capability dac_read_search,")
    helpers.assert_not_contains(c, "capability fsetid,")
    for line in c.splitlines():
        assert line.strip() != "/ rw,", "Must not allow root filesystem read-write"


# ---- Laravel nginx config ----


def test_laravel_nginx_prefers_php_over_html():
    c = _read("runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2")
    helpers.assert_contains(c, "index index.php index.html;")


def test_laravel_nginx_uses_absolute_fastcgi_params():
    c = _read("runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2")
    helpers.assert_contains(c, "include /etc/nginx/fastcgi_params;")
    helpers.assert_not_contains(c, "include fastcgi_params;")


def test_laravel_nginx_uses_resolved_path_manifest():
    c = _read("runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2")
    helpers.assert_contains(c, "pid {{ paths.runtime_nginx_pid }}")
    helpers.assert_contains(c, "listen unix:{{ paths.runtime_nginx_socket }}")
    helpers.assert_contains(c, "root {{ paths.current_web_root }}")
    helpers.assert_contains(c, "{{ paths.runtime_socket_dir }}/")
    helpers.assert_not_contains(c, "/run/{{ project_name }}")
    helpers.assert_not_contains(c, "{{ project_root }}/current/{{ web_root }}")


def test_laravel_nginx_logs_under_runtime_socket_dir():
    c = _read("runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2")
    helpers.assert_contains(c, "error_log {{ paths.runtime_socket_dir }}/error.log")
    helpers.assert_contains(c, "access_log {{ paths.runtime_socket_dir }}/access.log")
    helpers.assert_not_contains(c, "access_log stderr")


# ---- Laravel build script ----


def test_laravel_build_script_has_err_trap():
    c = _read("runtimes/laravel/deployment/02_run_build.sh")
    helpers.assert_contains(c, "trap '")
    helpers.assert_contains(c, "ERR")
    helpers.assert_contains(c, "$LINENO")
    helpers.assert_contains(c, "$BASH_COMMAND")


def test_laravel_build_script_labels_each_phase():
    c = _read("runtimes/laravel/deployment/02_run_build.sh")
    for label in [
        "Installing Composer dependencies",
        "Entering Laravel maintenance mode",
        "Installing frontend dependencies",
        "Building frontend assets",
        "Running migrations",
        "Rebuilding Laravel caches",
    ]:
        helpers.assert_contains(c, label)
