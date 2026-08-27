from pathlib import Path
from types import SimpleNamespace

from bonesinfra.services.linux import cloudflared


def test_install_uses_conditional_atomic_key_download(monkeypatch):
    calls = []

    monkeypatch.setattr(cloudflared.files, "directory", lambda **kwargs: calls.append(("directory", kwargs)))
    monkeypatch.setattr(cloudflared.files, "download", lambda **kwargs: calls.append(("download", kwargs)))
    monkeypatch.setattr(cloudflared.files, "template", lambda **kwargs: calls.append(("template", kwargs)))
    monkeypatch.setattr(cloudflared.apt, "packages", lambda **kwargs: calls.append(("packages", kwargs)))

    cloudflared.install()

    assert [operation for operation, _kwargs in calls] == ["directory", "download", "template", "packages"]
    assert calls[1][1] == {
        "name": "Install Cloudflare package signing key",
        "src": "https://pkg.cloudflare.com/cloudflare-main.gpg",
        "dest": "/etc/apt/keyrings/cloudflare-main.gpg",
        "user": "root",
        "group": "root",
        "mode": "0644",
        "_sudo": True,
    }


def test_quick_tunnel_setup_renders_project_service_for_nginx_socket(monkeypatch):
    calls = []
    ctx = SimpleNamespace(
        app=SimpleNamespace(project_name="atlas"),
        paths_dict={
            "systemd_cloudflared_service": "/etc/systemd/system/atlas-cloudflared.service",
            "runtime_nginx_socket": "/run/atlas/nginx/nginx.sock",
        },
    )

    monkeypatch.setattr(cloudflared.files, "template", lambda **kwargs: calls.append(kwargs))
    monkeypatch.setattr(cloudflared.service, "register_service", lambda *_args, **kwargs: calls.append(kwargs))
    monkeypatch.setattr(cloudflared.systemd, "daemon_reload", lambda **kwargs: calls.append(kwargs))
    monkeypatch.setattr(
        cloudflared,
        "template_data",
        lambda _ctx, *, paths: {"project_name": "atlas", "runtime_user": "atlas", "paths": paths},
    )

    cloudflared.setup(ctx, ctx.paths_dict)

    assert calls[0]["dest"] == "/etc/systemd/system/atlas-cloudflared.service"
    assert calls[0]["src"].endswith("systemd/cloudflared.service.j2")
    assert calls[1]["name"] == "cloudflared"


def test_quick_tunnel_unit_orders_network_and_allows_only_required_families():
    template = Path(cloudflared.ASSETS_DIR / "systemd/cloudflared.service.j2").read_text()

    assert "After=network-online.target {{ project_name }}-nginx.service" in template
    assert "Wants=network-online.target" in template
    assert "StartLimitIntervalSec=0" in template
    assert "RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6" in template
    assert "ProtectClock=yes" in template
    assert "ProtectKernelLogs=yes" in template
    assert "RestrictSUIDSGID=yes" in template
    assert "Restart=always" in template


def test_quick_tunnel_removal_unregisters_and_deletes_the_project_unit(monkeypatch):
    calls = []
    ctx = SimpleNamespace(app=SimpleNamespace(project_name="atlas"))
    paths = {
        "systemd_site_target_requires": "/etc/systemd/system/atlas.target.requires",
        "systemd_cloudflared_service": "/etc/systemd/system/atlas-cloudflared.service",
    }

    monkeypatch.setattr(cloudflared.systemd, "service", lambda **kwargs: calls.append(kwargs))
    monkeypatch.setattr(cloudflared.server, "shell", lambda **kwargs: calls.append(kwargs))
    monkeypatch.setattr(cloudflared.systemd, "daemon_reload", lambda **kwargs: calls.append(kwargs))

    cloudflared.remove(ctx, paths)

    assert calls[0]["service"] == "atlas-cloudflared.service"
    assert "atlas.target.requires/atlas-cloudflared.service" in calls[1]["commands"][0]
    assert "atlas-cloudflared.service" in calls[1]["commands"][1]
