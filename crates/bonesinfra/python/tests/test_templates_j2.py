"""Security invariants in Jinja2 templates — negative assertions only."""

import jinja2

from . import helpers

N = helpers.SRC_DIR / "bonesinfra"


def _read(name):
    return helpers.read(N / name)


def test_every_template_parses_as_jinja():
    """All .j2 assets must survive Jinja parsing. Catches Go-template syntax
    (podman/docker --format '{{.Field}}') left unescaped inside Jinja templates,
    which otherwise only explodes at provision time on a real server."""
    env = jinja2.Environment(autoescape=True)
    templates = sorted(N.rglob("*.j2"))
    assert templates, "no .j2 templates found — SRC_DIR layout changed?"
    for path in templates:
        try:
            env.parse(helpers.read(path))
        except jinja2.TemplateSyntaxError as err:
            raise AssertionError(f"{path.relative_to(N)} L{err.lineno}: {err.message}") from err


def test_default_deny_config_is_default_deny_only():
    """The default-deny vhost must never proxy, serve files, or reach project state."""
    c = _read("assets/nginx/default-deny.conf.j2")
    helpers.assert_contains(c, "return 444;")
    helpers.assert_not_contains(c, "proxy_pass")
    helpers.assert_not_contains(c, "root ")
    helpers.assert_not_contains(c, "runtime_nginx_socket")
    helpers.assert_not_contains(c, "runtime_socket_dir")
    helpers.assert_not_contains(c, "current_web_root")


def test_common_apparmor_profile_uses_configurable_network():
    """AppArmor network rule must come from the template variable, not be hardcoded."""
    c = _read("runtimes/common/assets/app-profile.j2")
    helpers.assert_contains(c, '{{ apparmor_network | default("network unix stream,") }}')
    helpers.assert_not_contains(c, "{{ paths.current }}/** r,")


def test_site_nginx_service_runtime_dir_is_traversable():
    """Per-site nginx RuntimeDirectory must be 0711 so www-data can reach the socket.
    Regression: 0750 caused 502 on every public request after the per-site nginx
    layout moved the socket under /run/<project>/nginx/."""
    c = _read("assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "RuntimeDirectoryMode=0711")
    helpers.assert_not_contains(c, "RuntimeDirectoryMode=0750")


def test_app_service_runtime_dir_stays_private():
    """App runtime dirs stay 0750 — only the per-site nginx (same runtime user)
    needs to reach app sockets, so no world traversal is required."""
    c = _read("runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "RuntimeDirectoryMode=0750")


def test_site_nginx_service_conditionally_restricts_ip_to_loopback():
    """TCP-mode nginx must opt in to loopback restriction; the default must not
    hardcode it so unix-socket runtimes don't accidentally inherit it."""
    c = _read("assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "{% if nginx_ip_loopback_only %}IPAddressDeny=any")
    helpers.assert_contains(c, "IPAddressAllow=localhost")
