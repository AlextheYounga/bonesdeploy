from pathlib import Path
from types import SimpleNamespace

import pytest

from bonesinfra.services.linux import cloudflared


def _context(backend="native", compose_port=8080):
    return SimpleNamespace(
        app=SimpleNamespace(project_name="atlas"),
        runtime=SimpleNamespace(backend=backend, compose_port=compose_port),
        paths_dict={
            "systemd_cloudflared_service": "/etc/systemd/system/atlas-cloudflared.service",
            "nginx_cloudflared_site_available": "/etc/nginx/sites-available/bonesdeploy-cloudflared-atlas.conf",
            "nginx_cloudflared_site_enabled": "/etc/nginx/sites-enabled/bonesdeploy-cloudflared-atlas.conf",
            "runtime_nginx_socket": "/run/atlas/nginx/nginx.sock",
            "systemd_site_target_requires": "/etc/systemd/system/atlas.target.requires",
        },
    )


def test_port_for_project_is_stable_and_unprivileged():
    assert cloudflared.port_for("atlas") == cloudflared.port_for("atlas")
    assert 1024 < cloudflared.port_for("atlas") < 65536


def test_quick_tunnel_rejects_compose_without_managed_nginx():
    with pytest.raises(ValueError, match="managed nginx ingress"):
        cloudflared.validate_supported(_context(backend="docker", compose_port=None))


def test_quick_tunnel_start_installs_loopback_route_and_independent_service(monkeypatch):
    calls = []
    ctx = _context()
    monkeypatch.setattr(cloudflared, "install", lambda: calls.append("install"))
    monkeypatch.setattr(
        cloudflared,
        "render",
        lambda _name, _src, dest, **kwargs: calls.append(("render", {"dest": dest, **kwargs})),
    )
    monkeypatch.setattr(cloudflared.server, "script", lambda **kwargs: calls.append(("script", kwargs)))
    monkeypatch.setattr(cloudflared.systemd, "service", lambda **kwargs: calls.append(("service", kwargs)))
    monkeypatch.setattr(cloudflared.systemd, "daemon_reload", lambda **_: calls.append("reload-systemd"))
    monkeypatch.setattr(cloudflared.server, "shell", lambda **kwargs: calls.append(("shell", kwargs)))
    monkeypatch.setattr(cloudflared, "template_data", lambda _ctx, *, paths: {"project_name": "atlas", "paths": paths})
    monkeypatch.setattr(cloudflared.files, "template", lambda **kwargs: calls.append(("template", kwargs)))

    cloudflared.start(ctx, ctx.paths_dict)

    assert calls[0] == "install"
    route = next(call for call in calls if call[0] == "render")
    assert route[1]["dest"] == ctx.paths_dict["nginx_cloudflared_site_available"]
    assert route[1]["cloudflared_port"] == cloudflared.port_for("atlas")
    activation = next(call[1] for call in calls if call[0] == "script")
    assert activation["args"] == (
        ctx.paths_dict["nginx_cloudflared_site_available"],
        ctx.paths_dict["nginx_cloudflared_site_enabled"],
    )
    assert "register_service" not in str(calls)
    target_cleanup = next(call[1]["commands"][0] for call in calls if call[0] == "shell")
    assert "atlas.target.requires/atlas-cloudflared.service" in target_cleanup
    start = [call for call in calls if call[0] == "service"][-1][1]
    assert start["enabled"] is True
    assert start["restarted"] is True


def test_quick_tunnel_unit_uses_supported_loopback_http_origin_and_orders_nginx():
    template = Path(cloudflared.ASSETS_DIR / "systemd/cloudflared.service.j2").read_text()

    assert "After=network-online.target nginx.service {{ project_name }}-nginx.service" in template
    assert "Requires=nginx.service {{ project_name }}-nginx.service" in template
    assert "--url http://127.0.0.1:{{ cloudflared_port }}" in template
    assert "PartOf={{ project_name }}.target" not in template
    assert "{{ project_name }}.target.requires" not in template

    route = Path(cloudflared.ASSETS_DIR / "nginx/cloudflared.conf.j2").read_text()
    assert "listen 127.0.0.1:{{ cloudflared_port }} default_server;" in route
    assert "proxy_pass http://unix:{{ paths.runtime_nginx_socket }};" in route
    assert "proxy_set_header X-Forwarded-Proto https;" in route


def test_quick_tunnel_stop_removes_unit_and_both_route_paths(monkeypatch):
    calls = []
    ctx = _context()
    monkeypatch.setattr(cloudflared.systemd, "service", lambda **kwargs: calls.append(("service", kwargs)))
    monkeypatch.setattr(cloudflared.server, "shell", lambda **kwargs: calls.append(("shell", kwargs)))
    monkeypatch.setattr(cloudflared.systemd, "daemon_reload", lambda **_: calls.append("reload-systemd"))
    monkeypatch.setattr(cloudflared, "validate_config", lambda *_: calls.append("validate"))

    cloudflared.stop(ctx, ctx.paths_dict)

    command = next(call[1]["commands"][0] for call in calls if call[0] == "shell")
    assert "systemctl disable --now" in command
    assert ctx.paths_dict["systemd_cloudflared_service"] in command
    assert ctx.paths_dict["nginx_cloudflared_site_available"] in command
    assert ctx.paths_dict["nginx_cloudflared_site_enabled"] in command
    assert "atlas.target.requires/atlas-cloudflared.service" in command
