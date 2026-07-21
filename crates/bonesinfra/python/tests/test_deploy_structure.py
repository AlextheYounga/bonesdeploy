"""Operation ordering and separation of concerns in deploy plans."""

from jinja2 import Template

from . import helpers

SETUP_PLAN = helpers.SRC_DIR / "bonesinfra/deploys/setup/plan.py"
SETUP_PACKAGES = helpers.SRC_DIR / "bonesinfra/deploys/setup/packages.py"
SETUP_USERS = helpers.SRC_DIR / "bonesinfra/deploys/setup/users.py"
BUILD_SLICE_DROPIN = helpers.SRC_DIR / "bonesinfra/assets/systemd/bonesdeploy-build.slice.j2"
SETUP_DIRECTORIES = helpers.SRC_DIR / "bonesinfra/deploys/setup/directories.py"
SETUP_PLACEHOLDER = helpers.SRC_DIR / "bonesinfra/deploys/setup/placeholder.py"
SETUP_FIREWALL = helpers.SRC_DIR / "bonesinfra/deploys/setup/firewall.py"
SETUP_FAIL2BAN = helpers.SRC_DIR / "bonesinfra/deploys/setup/fail2ban.py"
SETUP_KERNEL_HARDENING = helpers.SRC_DIR / "bonesinfra/deploys/setup/kernel_hardening.py"
DISABLE_ALGIF = helpers.SRC_DIR / "bonesinfra/assets/modprobe/disable-algif.conf.j2"
SETUP_UNATTENDED_UPGRADES = helpers.SRC_DIR / "bonesinfra/deploys/setup/unattended_upgrades.py"
SETUP_BONESREMOTE = helpers.SRC_DIR / "bonesinfra/deploys/setup/bonesremote.py"
SETUP_SUDOERS = helpers.SRC_DIR / "bonesinfra/deploys/setup/sudoers.py"
HELPERS_NEOVIM = helpers.SRC_DIR / "bonesinfra/deploys/helpers/neovim.py"
RUNTIME_PLAN = helpers.SRC_DIR / "bonesinfra/deploys/runtime/plan.py"
RUNTIME_PACKAGES = helpers.SRC_DIR / "bonesinfra/deploys/runtime/packages.py"
RUNTIME_APPARMOR = helpers.SRC_DIR / "bonesinfra/deploys/runtime/apparmor.py"
RUNTIME_NGINX = helpers.SRC_DIR / "bonesinfra/deploys/runtime/nginx.py"
RUNTIME_DOCTOR = helpers.SRC_DIR / "bonesinfra/deploys/runtime/doctor.py"
RUNTIME_TEMPLATE = helpers.SRC_DIR / "bonesinfra/deploys/runtime/template_runtime.py"
SSL_PLAN = helpers.SRC_DIR / "bonesinfra/deploys/ssl/plan.py"
LARAVEL_DEPLOY = helpers.SRC_DIR / "bonesinfra/runtimes/laravel/deploy.py"

SCRIPTS_DIR = helpers.SRC_DIR / "bonesinfra/assets/scripts"
SCRIPT_NEOVIM_CONFIG = SCRIPTS_DIR / "install-neovim-config.sh"
SCRIPT_VERIFY_PODMAN = SCRIPTS_DIR / "verify-rootless-podman.sh.j2"
SCRIPT_RESOURCE_LIMITS = SCRIPTS_DIR / "apply-build-resource-limits.sh.j2"


# ---- setup plan ----


def test_setup_plan_calls_all_steps():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_contains(c, "packages.install_system")
    helpers.assert_contains(c, "users.install_rust")
    helpers.assert_contains(c, "users.ensure_users_and_groups")
    helpers.assert_contains(c, "directories.setup_repo_and_project")
    helpers.assert_contains(c, "placeholder.seed")
    helpers.assert_contains(c, "firewall.configure")
    helpers.assert_contains(c, "fail2ban.configure")
    helpers.assert_contains(c, "kernel_hardening.configure")
    helpers.assert_contains(c, "unattended_upgrades.configure")
    helpers.assert_contains(c, "users.install_authorized_key")
    helpers.assert_contains(c, "bonesremote.install")
    helpers.assert_contains(c, "sudoers.install")


def test_setup_plan_uses_base_packages():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_contains(c, "BASE_SYSTEM_PACKAGES")


def test_setup_plan_ordering():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_ordering(
        c,
        "packages.install_system",
        "users.install_rust",
        "users.ensure_users_and_groups",
    )


def test_setup_installs_sudoers_in_bonesinfra():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_contains(c, "sudoers.install")
    c = helpers.read(SETUP_BONESREMOTE)
    helpers.assert_not_contains(c, "bonesremote init")
    c = helpers.read(SETUP_SUDOERS)
    helpers.assert_contains(c, "visudo")
    helpers.assert_contains(c, 'mode="0440"')
    helpers.assert_contains(c, "bonesremote_global_link")


def test_setup_uses_resolved_placeholder_paths():
    c1 = helpers.read(SETUP_DIRECTORIES)
    helpers.assert_contains(c1, "placeholder_web_root")
    c2 = helpers.read(SETUP_PLACEHOLDER)
    helpers.assert_contains(c2, "placeholder_index")


def test_setup_seeds_bare_repo_post_receive_hook():
    c = helpers.read(SETUP_DIRECTORIES)
    helpers.assert_contains(c, '"Install bare repo post-receive hook"')
    helpers.assert_contains(c, "dest=f\"{paths['repo']}/hooks/post-receive\"")
    helpers.assert_contains(c, "user=DEPLOY_USER")
    helpers.assert_contains(c, 'mode="0755"')


def test_bare_repo_init_sets_default_branch():
    c = helpers.read(SETUP_DIRECTORIES)
    helpers.assert_contains(c, '"Set bare repo default branch"')
    helpers.assert_contains(c, "git --git-dir")
    helpers.assert_contains(c, "symbolic-ref HEAD refs/heads/{ctx.app.deploy.branch}")


def test_post_receive_hook_execs_bonesremote():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/assets/hooks/post-receive")
    helpers.assert_contains(c, "set -euo pipefail")
    helpers.assert_contains(c, "SITE=${SITE%.git}")
    helpers.assert_contains(c, 'exec sudo bonesremote hook post-receive --site "$SITE"')


def test_setup_avoids_usermod_for_existing_runtime_user():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "host.get_fact(Users)")
    helpers.assert_contains(c, "gpasswd -a")


def test_git_not_in_runtime_group():
    c = helpers.read(SETUP_USERS)
    helpers.assert_not_contains(c, "_ensure_group_membership(DEPLOY_USER, ctx.runtime.runtime_group)")
    helpers.assert_not_contains(c, "ctx.runtime.release_group")


def test_shared_is_runtime_owned_private_state():
    c = helpers.read(SETUP_DIRECTORIES)
    shared_block = c.split('path=paths["shared"]')[1].split(")")[0]
    helpers.assert_contains(shared_block, "user=ctx.runtime.runtime_user")
    helpers.assert_contains(shared_block, "group=ctx.runtime.runtime_group")
    helpers.assert_contains(shared_block, 'mode="0750"')


def test_releases_are_root_owned_group_readable():
    c = helpers.read(SETUP_DIRECTORIES)
    releases_block = c.split('path=paths["releases"]')[1].split(")")[0]
    helpers.assert_contains(releases_block, 'user="root"')
    helpers.assert_contains(releases_block, "group=ctx.runtime.runtime_group")
    helpers.assert_contains(releases_block, 'mode="2750"')


def test_setup_deploy_user_commands_set_user_home():
    c = helpers.read(SETUP_DIRECTORIES)
    helpers.assert_contains(c, "XDG_CONFIG_HOME={home}/.config")
    helpers.assert_contains(c, "getent passwd")


# ---- Firewall ----


def test_setup_firewall_handles_ssh_cidrs():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "ssh_cidrs")


def test_setup_firewall_filters_ssh_from_allowed_ports():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, 'port == "ssh"')
    helpers.assert_contains(c, "continue")


def test_setup_firewall_resolves_port_aliases():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "port_aliases.get(port, port)")


def test_setup_firewall_sets_default_policies():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "ufw --force default")
    helpers.assert_contains(c, "firewall_default_incoming_policy")
    helpers.assert_contains(c, "firewall_default_outgoing_policy")


def test_setup_firewall_gated_by_enabled_flag():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "firewall_enabled")


def test_setup_firewall_status_gated_by_show_status():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "ufw status verbose")
    helpers.assert_contains(c, "firewall_show_status")


def test_setup_fail2ban_configures_sshd_jail_and_service():
    c = helpers.read(SETUP_FAIL2BAN)
    helpers.assert_contains(c, '"/etc/fail2ban/jail.local"')
    helpers.assert_contains(c, '"fail2ban"')
    helpers.assert_contains(c, "ssh_port=int(ctx.app.server.port)")


def test_setup_unattended_upgrades_installs_apt_configs():
    c = helpers.read(SETUP_UNATTENDED_UPGRADES)
    helpers.assert_contains(c, '"/etc/apt/apt.conf.d/20auto-upgrades"')
    helpers.assert_contains(c, '"/etc/apt/apt.conf.d/50unattended-upgrades"')


def test_setup_guarantees_copy_fail_module_is_disabled():
    plan = helpers.read(SETUP_PLAN)
    hardening = helpers.read(SETUP_KERNEL_HARDENING)
    modprobe_config = helpers.read(DISABLE_ALGIF)

    helpers.assert_ordering(plan, "packages.install_system", "kernel_hardening.configure")
    helpers.assert_contains(hardening, '"/etc/modprobe.d/disable-algif.conf"')
    helpers.assert_contains(hardening, "rmmod algif_aead")
    helpers.assert_not_contains(hardening, "rmmod algif_aead 2>/dev/null || true")
    helpers.assert_contains(modprobe_config, "install algif_aead /bin/false")


def test_helpers_neovim_installs_config_from_repo():
    py_source = helpers.read(HELPERS_NEOVIM)
    helpers.assert_contains(py_source, "install-neovim-config.sh")

    script = helpers.read(SCRIPT_NEOVIM_CONFIG)
    helpers.assert_contains(script, "https://github.com/AlextheYounga/myneovim.git")
    helpers.assert_contains(script, "git clone --depth=1")
    helpers.assert_contains(script, "reset --hard FETCH_HEAD")


# ---- runtime plan ----


def test_runtime_plan_calls_all_steps():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_contains(c, "packages.install_apt")
    helpers.assert_contains(c, "apparmor.setup")
    helpers.assert_contains(c, "nginx.setup")
    helpers.assert_contains(c, "template_runtime.load")
    helpers.assert_contains(c, "nginx.start_services")
    helpers.assert_not_contains(c, "doctor.run")


def test_runtime_applies_apparmor_and_nginx():
    c = helpers.read(RUNTIME_APPARMOR)
    helpers.assert_contains(c, "apparmor_parser -r")
    helpers.assert_contains(c, "aa-enforce")
    helpers.assert_contains(c, "apparmor_enabled_param")
    c2 = helpers.read(RUNTIME_NGINX)
    helpers.assert_contains(c2, "per-site nginx")


def test_common_apparmor_enforces_after_load():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/apparmor.py")
    helpers.assert_ordering(
        c,
        "apparmor_parser -r",
        "aa-enforce",
    )


def test_common_service_verifies_profile_attached():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/service.py")
    helpers.assert_contains(c, "validation.verify_profile_attached")
    helpers.assert_contains(c, "apparmor_profile_name=None")


def test_common_validation_verifies_proc_attr_current():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/validation.py")
    helpers.assert_contains(c, "def verify_profile_attached")
    helpers.assert_contains(c, "attr/current")
    helpers.assert_contains(c, "MainPID")
    helpers.assert_contains(c, "systemctl status")
    helpers.assert_contains(c, "journalctl -u")


def test_app_service_uses_per_service_runtime_directory_leaf():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "RuntimeDirectory={{ project_name }}/{{ runtime_name }}")


def test_site_nginx_service_uses_nginx_runtime_directory_leaf():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "RuntimeDirectory={{ project_name }}/nginx")
    helpers.assert_contains(c, "ReadWritePaths={{ paths.runtime_nginx_dir }}")


def test_project_nginx_profile_grants_nginx_dir_and_app_sockets():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "{{ paths.runtime_nginx_dir }}/ rw,")
    helpers.assert_contains(c, "{{ paths.runtime_nginx_dir }}/** rwk,")
    helpers.assert_contains(c, "{{ paths.runtime_socket_dir }}/*/*.sock rw,")
    helpers.assert_not_contains(c, "{{ paths.runtime_socket_dir }}/** rwk,")


def test_runtime_plan_ordering():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_ordering(
        c,
        "packages.install_apt",
        "nginx.setup",
        "template_runtime.load",
        "nginx.start_services",
    )


def test_runtime_socket_dir_runtime_user_owned():
    c = helpers.read(RUNTIME_NGINX)
    helpers.assert_contains(c, 'path=paths["runtime_socket_dir"]')
    helpers.assert_contains(c, 'path=paths["runtime_nginx_dir"]')
    helpers.assert_contains(c, "user=ctx.runtime.runtime_user")
    helpers.assert_contains(c, "group=ctx.runtime.runtime_group")
    # 0711: system nginx (www-data) must traverse /run/<project>/ and
    # /run/<project>/nginx/ to reach the per-site nginx socket. 0750 causes 502.
    helpers.assert_contains(c, 'mode="0711"')


def test_runtime_uses_template_runtime():
    c = helpers.read(RUNTIME_TEMPLATE)
    helpers.assert_contains(c, "get_runtime(template)")


def test_setup_installs_podman_networking():
    c = helpers.read(SETUP_PACKAGES)
    helpers.assert_contains(c, '"podman"')
    helpers.assert_contains(c, '"netavark"')
    helpers.assert_contains(c, '"aardvark-dns"')
    helpers.assert_contains(c, '"passt"')
    helpers.assert_contains(c, '"slirp4netns"')
    helpers.assert_contains(c, '"uidmap"')
    helpers.assert_contains(c, '"fuse-overlayfs"')
    helpers.assert_contains(c, '"dbus-user-session"')


def test_setup_creates_rootless_build_user_and_pseudo_home():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "build_user_for(ctx.app.project_name)")
    helpers.assert_contains(c, "group=build_group")
    helpers.assert_contains(c, "home=build_home")
    helpers.assert_contains(c, "create_home=True")
    helpers.assert_contains(c, 'shell="/usr/sbin/nologin"')
    helpers.assert_contains(c, "path=BUILD_USER_HOME_ROOT")
    helpers.assert_contains(c, "path=build_home")
    helpers.assert_contains(c, 'mode="0700"')
    helpers.assert_ordering(c, "path=BUILD_USER_HOME_ROOT", 'name="Ensure build user exists"')
    build_user = c.split('name="Ensure build user exists"')[1].split(")")[0]
    helpers.assert_not_contains(build_user, "system=True")


def test_setup_keeps_runtime_user_homeless_and_non_login():
    c = helpers.read(SETUP_USERS)
    runtime_user = c.split('name="Ensure runtime user exists with groups"')[1].split(")")[0]

    helpers.assert_contains(runtime_user, 'home="/nonexistent"')
    helpers.assert_contains(runtime_user, 'shell="/usr/sbin/nologin"')
    helpers.assert_contains(runtime_user, "create_home=False")


def test_setup_enables_linger_for_build_user():
    py_source = helpers.read(SETUP_USERS)
    helpers.assert_contains(py_source, "loginctl enable-linger")
    helpers.assert_contains(py_source, "systemctl start user@$(id -u")
    helpers.assert_contains(py_source, "verify-rootless-podman.sh.j2")
    helpers.assert_contains(py_source, "_chdir=build_home")

    script = helpers.read(SCRIPT_VERIFY_PODMAN)
    helpers.assert_contains(script, "getsubids")
    helpers.assert_contains(script, "getsubids -g")
    helpers.assert_contains(script, 'test -S "/run/user/')
    helpers.assert_contains(script, "podman info")
    helpers.assert_not_contains(script, "Delegate=")
    helpers.assert_not_contains(script, "cgroup.controllers")
    helpers.assert_not_contains(script, "python3 -c")


def test_setup_limits_the_numeric_build_user_slice_idempotently():
    py_source = helpers.read(SETUP_USERS)
    dropin = helpers.read(BUILD_SLICE_DROPIN)

    helpers.assert_contains(py_source, "host.get_fact(Cpus)")
    helpers.assert_contains(py_source, "apply-build-resource-limits.sh.j2")
    helpers.assert_not_contains(py_source, "Delegate=")
    helpers.assert_not_contains(py_source, "cgroupshared")

    script = helpers.read(SCRIPT_RESOURCE_LIMITS)
    helpers.assert_contains(script, "user-${uid}.slice")
    helpers.assert_contains(script, "bonesdeploy-build.conf")
    helpers.assert_contains(script, "cmp -s")
    helpers.assert_contains(script, "systemctl daemon-reload")
    helpers.assert_contains(script, "systemctl set-property --runtime")
    helpers.assert_not_contains(py_source, "Delegate=")
    helpers.assert_contains(dropin, "CPUQuota={{ cpu_quota }}")
    helpers.assert_contains(dropin, "MemoryHigh={{ memory_high }}")
    helpers.assert_contains(dropin, "MemoryMax={{ memory_max }}")
    assert Template(dropin).render(cpu_quota="300%", memory_high="80%", memory_max="80%") == (
        "[Slice]\nCPUQuota=300%\nMemoryHigh=80%\nMemoryMax=80%"
    )


# ---- ssl plan ----


def test_ssl_uses_certbot():
    c = helpers.read(SSL_PLAN)
    helpers.assert_contains(c, "certbot certonly")
    helpers.assert_contains(c, "ssl_enabled")


def test_setup_installs_openssl_for_nginx_default_deny_cert():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/setup/packages.py")
    helpers.assert_contains(c, '"openssl"')


def test_runtime_nginx_falls_back_when_domain_empty():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/runtime/nginx.py")
    helpers.assert_contains(c, "ctx.app.dns.domain or ctx.app.dns.preview_domain")
    helpers.assert_contains(
        c,
        'raise ValueError("domain or preview_domain is required for nginx config")',
    )


def test_ssl_requires_real_domain_for_router_render():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/ssl/plan.py")
    helpers.assert_not_contains(c, 'raise ValueError("domain is required for ssl nginx config")')
    helpers.assert_contains(c, "nginx_server_name = ctx.app.dns.domain")


def test_ssl_uses_acme_webroot():
    c = helpers.read(SSL_PLAN)
    helpers.assert_contains(c, "acme_webroot")


# ---- Laravel-specific ordering ----


def test_laravel_validates_php_fpm_before_start():
    c = helpers.read(LARAVEL_DEPLOY)
    helpers.assert_ordering(
        c,
        "php_repo.add_php_apt_source",
        "php_packages.install_php",
    )


def test_laravel_validates_php_fpm_before_reload():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/php_fpm.py")
    helpers.assert_ordering(
        c,
        "cleanup_orphaned_pools(ctx, php_version)",
        "php_fpm_pool.render_pool",
        "php_fpm_pool.validate_php_fpm",
        "php_fpm_pool.reload_php_fpm",
    )


def test_laravel_nginx_validates_without_creating_runtime_dir():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/nginx.py")
    helpers.assert_ordering(
        c,
        "laravel-site-nginx.conf.j2",
        "nginx -t",
    )
    helpers.assert_not_contains(c, "runtime_socket_dir")


# ---- Runtime directory traversal for system nginx (www-data) ----


def test_runtime_socket_dir_is_traversable_by_system_nginx():
    """Regression: /run/<project>/ must be 0711, not 0750, so system nginx
    (www-data) can traverse it to reach /run/<project>/nginx/nginx.sock.

    0750 caused 502 on every public request after the per-site nginx layout
    change moved the socket under /run/<project>/nginx/.
    """
    runtime_nginx = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/runtime/nginx.py")
    # Both runtime dir mkdirs must use 0711; the conf dir (0750) is unrelated.
    socket_dir_block = runtime_nginx.split('path=paths["runtime_socket_dir"]')[1].split(")")[0]
    helpers.assert_contains(socket_dir_block, 'mode="0711"')
    nginx_dir_block = runtime_nginx.split('path=paths["runtime_nginx_dir"]')[1].split(")")[0]
    helpers.assert_contains(nginx_dir_block, 'mode="0711"')

    common_paths = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/paths.py")
    helpers.assert_contains(common_paths, 'mode="0711"')
    helpers.assert_not_contains(common_paths, 'mode="0750"')

    common_nginx = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/nginx.py")
    helpers.assert_contains(common_nginx, 'mode="0711"')
    helpers.assert_not_contains(common_nginx, 'mode="0750"')


def test_site_nginx_service_runtime_dir_is_traversable():
    """The per-site nginx RuntimeDirectory must be 0711 so www-data can
    traverse into /run/<project>/nginx/ to reach the socket."""
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "RuntimeDirectoryMode=0711")
    helpers.assert_not_contains(c, "RuntimeDirectoryMode=0750")


def test_app_service_runtime_dir_stays_private():
    """App runtime dirs stay 0750 — only the per-site nginx (same runtime
    user) needs to reach app sockets, so no world traversal is required."""
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "RuntimeDirectoryMode=0750")


def test_runtime_nginx_reloads_after_config_change():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/runtime/nginx.py")
    helpers.assert_ordering(
        c,
        "Enable router nginx site",
        "validate_config",
        "Ensure nginx service is enabled and started",
        'remove_direct_boot(ctx, "nginx")',
        "Enable and restart site systemd target",
        "systemctl reload nginx",
    )


def test_runtime_nginx_installs_default_deny_before_validation():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/runtime/nginx.py")
    helpers.assert_ordering(c, "install_default_deny_server", "validate_config")


def test_ssl_installs_default_deny_before_validation():
    c = helpers.read(SSL_PLAN)
    helpers.assert_ordering(c, "install_default_deny_server", "router.conf.j2", "validate_config")


def test_ssl_installs_default_deny_once():
    c = helpers.read(SSL_PLAN)
    assert c.count("install_default_deny_server") == 1


def test_runtime_plan_derives_nginx_address_families_from_template():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_contains(c, "nginx_address_families")
    helpers.assert_contains(c, "nginx_address_families=nginx_address_families")


def test_runtime_plan_derives_nginx_apparmor_network_from_template():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_contains(c, "nginx_apparmor_network")
    helpers.assert_contains(c, "nginx_apparmor_network=nginx_apparmor_network")


def test_runtime_plan_passes_tcp_values_when_tcp_runtime():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_contains(c, '"network inet stream,"')
    helpers.assert_contains(c, '"AF_UNIX AF_INET"')


def test_runtime_plan_defaults_to_unix_only():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_contains(c, '"network unix stream,"')
    helpers.assert_contains(c, '"AF_UNIX"')


def test_runtime_plan_passes_ip_loopback_only_to_nginx_setup():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_contains(c, "nginx_ip_loopback_only=uses_tcp")


def test_runtime_plan_imports_get_runtime():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_contains(c, "from bonesinfra.runtimes import get_runtime")


def test_runtime_plan_checks_uses_tcp_and_is_static():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_contains(c, "USES_TCP")
    helpers.assert_contains(c, '"is_static"')


def test_apparmor_setup_accepts_nginx_apparmor_network():
    c = helpers.read(RUNTIME_APPARMOR)
    helpers.assert_contains(c, "nginx_apparmor_network")
    helpers.assert_contains(c, "nginx_apparmor_network=nginx_apparmor_network")


def test_nginx_setup_accepts_nginx_address_families_and_ip_loopback():
    c = helpers.read(RUNTIME_NGINX)
    helpers.assert_contains(c, "nginx_address_families")
    helpers.assert_contains(c, "nginx_ip_loopback_only")
    helpers.assert_contains(c, "nginx_address_families=nginx_address_families")
    helpers.assert_contains(c, "nginx_ip_loopback_only=nginx_ip_loopback_only")
