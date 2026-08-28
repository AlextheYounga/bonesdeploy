import pytest

from bonesinfra.cli.commands.site.ssl import deploy_ssl, obtain_certificate
from bonesinfra.config.request import parse_request

from .helpers import make_site_request


@pytest.mark.parametrize("field", ["domain", "email"])
def test_request_rejects_invalid_ssl_boundary_values(field):
    request = make_site_request(**{field: "bad; value"})

    with pytest.raises(ValueError, match=rf"site\.{field}"):
        parse_request(request)


def test_certbot_arguments_are_shell_quoted(monkeypatch):
    calls = []
    monkeypatch.setattr("bonesinfra.cli.commands.site.ssl.server.shell", lambda **kwargs: calls.append(kwargs))
    ctx = parse_request(make_site_request())

    obtain_certificate(ctx, {"acme_webroot": "/srv/acme/web root;unsafe"})

    command = calls[0]["commands"][0]
    assert "--email ops@example.com" in command
    assert "-w '/srv/acme/web root;unsafe'" in command
    assert "-d example.com" in command


def test_ssl_flow_rejects_invalid_values_before_side_effects(monkeypatch):
    ctx = parse_request(make_site_request())
    ctx.app.dns.domain = "example.com; touch /tmp/pwned"
    mkdir_calls = []
    monkeypatch.setattr("bonesinfra.cli.commands.site.ssl.mkdir", lambda **kwargs: mkdir_calls.append(kwargs))

    with pytest.raises(ValueError, match=r"site\.domain"):
        deploy_ssl(ctx)
    assert mkdir_calls == []


def test_ssl_handoff_removes_quick_tunnel_after_ssl_router_is_ready(monkeypatch):
    calls = []
    ctx = parse_request(make_site_request(domain="example.com"))

    monkeypatch.setattr("bonesinfra.cli.commands.site.ssl.mkdir", lambda **_kwargs: calls.append("webroot"))
    monkeypatch.setattr(
        "bonesinfra.cli.commands.site.ssl.nginx_router.install_default_deny_server",
        lambda _paths: calls.append("default-deny"),
    )
    monkeypatch.setattr(
        "bonesinfra.cli.commands.site.ssl.nginx_router.render_router_config",
        lambda *_args, **kwargs: calls.append(f"router-{kwargs['stage']}"),
    )
    monkeypatch.setattr(
        "bonesinfra.cli.commands.site.ssl.obtain_certificate", lambda *_args: calls.append("certificate")
    )
    monkeypatch.setattr(
        "bonesinfra.cli.commands.site.ssl.cloudflared.remove",
        lambda *_args: calls.append("cloudflared-remove"),
    )
    monkeypatch.setattr(
        "bonesinfra.cli.commands.site.ssl.etckeeper.commit_changes",
        lambda *_args: calls.append("etckeeper-commit"),
    )

    deploy_ssl(ctx)

    assert calls == [
        "webroot",
        "default-deny",
        "router-certbot challenge",
        "certificate",
        "router-SSL enable",
        "cloudflared-remove",
        "etckeeper-commit",
    ]


def test_ssl_handoff_preserves_quick_tunnel_when_certificate_acquisition_fails(monkeypatch):
    calls = []
    ctx = parse_request(make_site_request(domain="example.com"))

    monkeypatch.setattr("bonesinfra.cli.commands.site.ssl.mkdir", lambda **_kwargs: calls.append("webroot"))
    monkeypatch.setattr(
        "bonesinfra.cli.commands.site.ssl.nginx_router.install_default_deny_server",
        lambda _paths: calls.append("default-deny"),
    )
    monkeypatch.setattr(
        "bonesinfra.cli.commands.site.ssl.nginx_router.render_router_config",
        lambda *_args, **kwargs: calls.append(f"router-{kwargs['stage']}"),
    )

    def fail_to_obtain_certificate(*_args):
        calls.append("certificate")
        raise RuntimeError("certbot failed")

    monkeypatch.setattr("bonesinfra.cli.commands.site.ssl.obtain_certificate", fail_to_obtain_certificate)
    monkeypatch.setattr(
        "bonesinfra.cli.commands.site.ssl.cloudflared.remove",
        lambda *_args: calls.append("cloudflared-remove"),
    )

    with pytest.raises(RuntimeError, match="certbot failed"):
        deploy_ssl(ctx)

    assert calls == ["webroot", "default-deny", "router-certbot challenge", "certificate"]
