"""Operation ordering and separation of concerns in setup.py / runtime.py / ssl.py."""

from . import helpers

SETUP = helpers.SRC_DIR / "setup.py"
RUNTIME = helpers.SRC_DIR / "runtime.py"
SSL = helpers.SRC_DIR / "ssl.py"


# ---- setup.py ----


def test_setup_excludes_apparmor():
    c = helpers.read(SETUP)
    helpers.assert_not_contains(c, "apparmor_parser")
    helpers.assert_not_contains(c, "apparmor_profile_name")


def test_setup_includes_firewall():
    c = helpers.read(SETUP)
    helpers.assert_contains(c, "ufw --force enable")


def test_setup_excludes_runtime_doctor():
    c = helpers.read(SETUP)
    helpers.assert_not_contains(c, "bonesremote doctor")


def test_setup_excludes_per_site_roles():
    c = helpers.read(SETUP)
    helpers.assert_not_contains(c, "bonesdeploy-nginx")
    helpers.assert_not_contains(c, "runtimes")
    helpers.assert_not_contains(c, "per-site nginx")


def test_setup_uses_single_package_manifest():
    c = helpers.read(SETUP)
    helpers.assert_contains(c, "BASE_SYSTEM_PACKAGES")


def test_setup_ordering_packages_before_rustup_before_users():
    c = helpers.read(SETUP)
    helpers.assert_ordering(
        c,
        "Install setup apt packages",
        "Install rustup and cargo",
        "Ensure deploy user exists",
    )


def test_setup_uses_resolved_placeholder_paths():
    c = helpers.read(SETUP)
    helpers.assert_contains(c, "placeholder_web_root")
    helpers.assert_contains(c, "placeholder_index")


# ---- Firewall checks ----


def test_setup_firewall_handles_ssh_cidrs():
    c = helpers.read(SETUP)
    helpers.assert_contains(c, "ssh_cidrs")


def test_setup_firewall_filters_ssh_from_allowed_ports():
    c = helpers.read(SETUP)
    helpers.assert_contains(c, 'port == "ssh"')
    helpers.assert_contains(c, "continue")


def test_setup_firewall_resolves_port_aliases():
    c = helpers.read(SETUP)
    helpers.assert_contains(c, "port_aliases.get(port, port)")


def test_setup_firewall_sets_default_policies():
    c = helpers.read(SETUP)
    helpers.assert_contains(c, "ufw --force default")
    helpers.assert_contains(c, "firewall_default_incoming_policy")
    helpers.assert_contains(c, "firewall_default_outgoing_policy")


def test_setup_firewall_gated_by_enabled_flag():
    c = helpers.read(SETUP)
    helpers.assert_contains(c, "firewall_enabled")


def test_setup_firewall_status_gated_by_show_status():
    c = helpers.read(SETUP)
    helpers.assert_contains(c, "ufw status verbose")
    helpers.assert_contains(c, "firewall_show_status")


# ---- runtime.py ----


def test_runtime_applies_apparmor_and_nginx():
    c = helpers.read(RUNTIME)
    helpers.assert_contains(c, "runtimes")
    helpers.assert_contains(c, "apparmor_parser -r")
    helpers.assert_contains(c, "per-site nginx")
    helpers.assert_contains(c, "aa-enforce")
    helpers.assert_contains(c, "apparmor_enabled_param")


def test_runtime_installs_packages_before_operations():
    c = helpers.read(RUNTIME)
    helpers.assert_ordering(
        c,
        "Install runtime apt packages",
        "get_runtime(template)",
    )


def test_runtime_excludes_ssl_logic():
    c = helpers.read(RUNTIME)
    helpers.assert_not_contains(c, "ssl_role")
    helpers.assert_not_contains(c, "certbot")


def test_runtime_socket_dir_runtime_user_owned():
    c = helpers.read(RUNTIME)
    helpers.assert_contains(c, 'path=paths["runtime_socket_dir"]')
    helpers.assert_contains(c, 'user=data["runtime_user"]')
    helpers.assert_contains(c, 'group=data["runtime_group"]')
    helpers.assert_contains(c, 'mode="0750"')


def test_runtime_derives_profile_from_project_name():
    c = helpers.read(RUNTIME)
    helpers.assert_contains(c, "data['project_name']")


# ---- ssl.py ----


def test_ssl_uses_certbot():
    c = helpers.read(SSL)
    helpers.assert_contains(c, "certbot certonly")
    helpers.assert_contains(c, "ssl_domain")


def test_ssl_excludes_apparmor():
    c = helpers.read(SSL)
    helpers.assert_not_contains(c, "apparmor_parser")


def test_ssl_excludes_per_site_nginx():
    c = helpers.read(SSL)
    helpers.assert_not_contains(c, '"per-site nginx"')


def test_ssl_excludes_runtimes():
    c = helpers.read(SSL)
    helpers.assert_not_contains(c, "runtimes")


def test_ssl_defines_nginx_inline():
    c = helpers.read(SSL)
    helpers.assert_contains(c, "nginx_server_name")
    helpers.assert_contains(c, "router.conf.j2")
    helpers.assert_contains(c, "nginx -t")


def test_ssl_uses_current_web_root():
    c = helpers.read(SSL)
    helpers.assert_contains(c, "current_web_root")


# ---- Laravel-specific ordering ----


def test_laravel_validates_php_fpm_before_start():
    c = helpers.read(helpers.SRC_DIR / "runtimes/laravel/laravel.py")
    helpers.assert_ordering(
        c,
        "--test --fpm-config",
        "Enable and start per-project PHP-FPM service",
    )


def test_laravel_creates_socket_dir_before_nginx_validation():
    c = helpers.read(helpers.SRC_DIR / "runtimes/laravel/laravel.py")
    helpers.assert_ordering(
        c,
        "Ensure runtime socket directory exists before nginx validation",
        "nginx -t",
    )
