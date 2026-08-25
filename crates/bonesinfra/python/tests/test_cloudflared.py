from types import SimpleNamespace

from bonesinfra.services.linux import cloudflared


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
