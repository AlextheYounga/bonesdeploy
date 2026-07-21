"""J2 template file existence and content assertions."""

from . import helpers

N = helpers.SRC_DIR / "bonesinfra"


def _read(name):
    return helpers.read(N / name)


# ---- Base AppArmor profile ----


# ---- Base nginx service template ----


# ---- Base nginx config ----


# ---- Router nginx config ----


def test_default_deny_config_is_default_deny_only():
    c = _read("assets/nginx/default-deny.conf.j2")
    helpers.assert_contains(c, "listen 80 default_server;")
    helpers.assert_contains(c, "listen 443 ssl default_server;")
    helpers.assert_contains(c, "server_name _;")
    helpers.assert_contains(c, "return 444;")
    helpers.assert_contains(c, "{{ paths.nginx_default_deny_ssl_certificate }}")
    helpers.assert_contains(c, "{{ paths.nginx_default_deny_ssl_certificate_key }}")
    # Mandatory dead-end: never proxy, serve files, or reach project state.
    helpers.assert_not_contains(c, "proxy_pass")
    helpers.assert_not_contains(c, "root ")
    helpers.assert_not_contains(c, "runtime_nginx_socket")
    helpers.assert_not_contains(c, "runtime_socket_dir")
    helpers.assert_not_contains(c, "current_web_root")


# ---- Laravel PHP-FPM pool config ----


# ---- Laravel nginx config ----


def test_laravel_nginx_has_larger_fastcgi_buffers():
    c = _read("runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2")
    helpers.assert_contains(c, "fastcgi_buffer_size 16k")
    helpers.assert_contains(c, "fastcgi_buffers 8 8k")
    helpers.assert_contains(c, "fastcgi_busy_buffers_size 16k")


# ---- Common app service template ----


def test_common_app_service_runs_as_runtime_user():
    c = _read("runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "User={{ runtime_user }}")
    helpers.assert_contains(c, "Group={{ runtime_group }}")
    helpers.assert_not_contains(c, "SupplementaryGroups={{ release_group }}")
    helpers.assert_contains(c, "WorkingDirectory={{ paths.current }}")
    helpers.assert_contains(c, "RuntimeDirectory={{ project_name }}/{{ runtime_name }}")
    helpers.assert_contains(c, "RuntimeDirectoryMode=0750")
    helpers.assert_contains(c, "EnvironmentFile=-{{ paths.conf_root }}/runtime.env")
    helpers.assert_contains(c, "ExecStart={{ runtime_exec }}")
    helpers.assert_contains(c, "AppArmorProfile={{ apparmor_profile_name }}")
    helpers.assert_contains(c, "PartOf={{ project_name }}.target")
    helpers.assert_contains(c, "Before={{ project_name }}.target")


def test_common_app_service_writes_to_runtime_and_logs_dirs():
    c = _read("runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "ReadOnlyPaths={{ paths.current }}")
    helpers.assert_contains(
        c,
        "ReadWritePaths={{ paths.runtime_socket_dir }}/{{ runtime_name }} "
        "{{ runtime_write_paths }} /var/log/bonesdeploy/{{ project_name }}",
    )
    helpers.assert_contains(c, "StandardOutput=journal")
    helpers.assert_contains(c, "StandardError=journal")
    helpers.assert_contains(c, "Restart=always")
    helpers.assert_contains(c, "RestartSec=2")


def test_site_target_is_a_stable_lifecycle_entrypoint():
    c = _read("runtimes/common/assets/site.target.j2")
    helpers.assert_contains(c, "Description=Lifecycle target for {{ project_name }}")
    helpers.assert_contains(c, "WantedBy=multi-user.target")


def test_site_nginx_service_participates_in_site_target():
    c = _read("assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "PartOf={{ project_name }}.target")
    helpers.assert_contains(c, "Before={{ project_name }}.target")


# ---- Common app nginx proxy template ----


def test_common_app_nginx_proxies_to_socket():
    c = _read("runtimes/common/assets/app-site-nginx.conf.j2")
    helpers.assert_contains(c, "proxy_pass {{ app_proxy_target }}")
    helpers.assert_contains(c, "proxy_set_header Host $host")
    helpers.assert_contains(c, "proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for")
    helpers.assert_contains(c, "proxy_set_header X-Forwarded-Proto $scheme")
    helpers.assert_contains(c, "proxy_set_header X-Real-IP $remote_addr")
    helpers.assert_contains(c, "proxy_http_version 1.1")
    helpers.assert_contains(c, 'proxy_set_header Connection ""')
    helpers.assert_contains(c, "proxy_buffer_size 16k")
    helpers.assert_contains(c, "proxy_buffers 8 8k")
    helpers.assert_contains(c, "proxy_busy_buffers_size 16k")


def test_router_nginx_has_larger_proxy_buffers():
    c = _read("assets/nginx/router.conf.j2")
    helpers.assert_contains(c, "proxy_buffer_size 16k")
    helpers.assert_contains(c, "proxy_buffers 8 8k")
    helpers.assert_contains(c, "proxy_busy_buffers_size 16k")


# ---- Common static nginx template ----


def test_common_static_nginx_serves_dist():
    c = _read("runtimes/common/assets/static-site-nginx.conf.j2")
    helpers.assert_contains(c, "root {{ paths.current }}/{{ static_root }}")
    helpers.assert_contains(c, "index index.html")
    helpers.assert_contains(c, "try_files $uri $uri/ /index.html")


# ---- Common app AppArmor profile ----


def test_common_apparmor_profile_uses_configurable_network():
    c = _read("runtimes/common/assets/app-profile.j2")
    helpers.assert_contains(c, '{{ apparmor_network | default("network unix stream,") }}')
    helpers.assert_contains(c, "{{ paths.releases }}/*/** r,")
    helpers.assert_not_contains(c, "{{ paths.current }}/** r,")


def test_fail2ban_template_enables_sshd_on_configured_port():
    c = _read("assets/fail2ban/jail.local.j2")
    helpers.assert_contains(c, "[sshd]")
    helpers.assert_contains(c, "enabled = true")
    helpers.assert_contains(c, "port = {{ ssh_port }}")


def test_unattended_upgrades_template_enables_periodic_security_updates():
    c = _read("assets/unattended-upgrades/20auto-upgrades.j2")
    helpers.assert_contains(c, 'APT::Periodic::Update-Package-Lists "1";')
    helpers.assert_contains(c, 'APT::Periodic::Unattended-Upgrade "1";')


def test_unattended_upgrades_template_uses_distro_variables():
    c = _read("assets/unattended-upgrades/50unattended-upgrades.j2")
    helpers.assert_contains(c, '"${distro_id}:${distro_codename}-security";')
    helpers.assert_contains(c, 'Unattended-Upgrade::Automatic-Reboot "false";')


def test_site_nginx_service_uses_configurable_address_families():
    c = _read("assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, 'RestrictAddressFamilies={{ nginx_address_families | default("AF_UNIX") }}')
    helpers.assert_not_contains(c, "RestrictAddressFamilies=AF_UNIX")


def test_site_nginx_service_conditionally_restricts_ip_to_loopback():
    c = _read("assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "{% if nginx_ip_loopback_only %}IPAddressDeny=any")
    helpers.assert_contains(c, "IPAddressAllow=localhost")


def test_project_nginx_profile_uses_configurable_network():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, '{{ nginx_apparmor_network | default("network unix stream,") }}')
    helpers.assert_not_contains(c, "\n  network unix stream,")
