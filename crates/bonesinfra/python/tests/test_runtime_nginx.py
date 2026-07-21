import pytest

from bonesinfra.deploys.runtime import nginx as runtime_nginx
from bonesinfra.domain.context import DeployContext


def _make_ctx(tmp_path, *, domain: str = "", preview_domain: str = "preview.example.com"):
    config_path = tmp_path / "bones.toml"
    config_path.write_text(
        f"""[app]
project_name = "lawsnipe"
[app.server]
host = "example.com"
[app.dns]
domain = "{domain}"
preview_domain = "{preview_domain}"
email = "ops@example.com"
"""
    )
    return DeployContext.from_files(str(config_path))


def _noop(*args, **kwargs):
    del args, kwargs


def test_runtime_nginx_uses_preview_domain_when_domain_is_empty(tmp_path, monkeypatch):
    ctx = _make_ctx(tmp_path, domain="", preview_domain="preview.example.com")
    paths = ctx.paths_dict
    calls = []

    monkeypatch.setattr(runtime_nginx, "mkdir", _noop)
    monkeypatch.setattr(runtime_nginx.service, "render_target", _noop)
    monkeypatch.setattr(runtime_nginx.service, "register_service", _noop)
    monkeypatch.setattr(runtime_nginx.files, "link", _noop)
    monkeypatch.setattr(runtime_nginx.server, "shell", _noop)
    monkeypatch.setattr(runtime_nginx.systemd, "daemon_reload", _noop)
    monkeypatch.setattr(runtime_nginx.nginx_safety, "install_default_deny_server", _noop)
    monkeypatch.setattr(runtime_nginx.nginx_safety, "validate_config", _noop)

    def fake_render(*args, **kwargs):
        calls.append((args, kwargs))

    monkeypatch.setattr(runtime_nginx, "render", fake_render)

    runtime_nginx.setup(ctx, paths)

    router_call = next(call for _, call in calls if "nginx_server_name" in call)
    assert router_call["nginx_server_name"] == "preview.example.com"
    assert router_call["preview_domain"] == "preview.example.com"


def test_runtime_nginx_requires_a_real_name(tmp_path, monkeypatch):
    ctx = _make_ctx(tmp_path, domain="", preview_domain="")
    paths = ctx.paths_dict

    monkeypatch.setattr(runtime_nginx, "mkdir", _noop)
    monkeypatch.setattr(runtime_nginx.service, "render_target", _noop)
    monkeypatch.setattr(runtime_nginx.service, "register_service", _noop)
    monkeypatch.setattr(runtime_nginx.files, "link", _noop)
    monkeypatch.setattr(runtime_nginx.server, "shell", _noop)
    monkeypatch.setattr(runtime_nginx.systemd, "daemon_reload", _noop)
    monkeypatch.setattr(runtime_nginx.nginx_safety, "install_default_deny_server", _noop)
    monkeypatch.setattr(runtime_nginx.nginx_safety, "validate_config", _noop)
    monkeypatch.setattr(runtime_nginx, "render", _noop)

    with pytest.raises(ValueError, match="domain or preview_domain"):
        runtime_nginx.setup(ctx, paths)


def test_runtime_nginx_migrates_site_service_to_target(monkeypatch):
    calls = []
    shell_calls = []
    ctx = type("Context", (), {"app": type("App", (), {"project_name": "shop"})()})()
    paths = {
        "systemd_site_nginx_service": "/etc/systemd/system/shop-nginx.service",
        "systemd_site_target": "/etc/systemd/system/shop.target",
    }
    monkeypatch.setattr(runtime_nginx.systemd, "service", lambda **kwargs: calls.append(kwargs))
    monkeypatch.setattr(runtime_nginx.server, "shell", lambda **kwargs: shell_calls.append(kwargs))

    runtime_nginx.start_services(ctx, paths)

    assert shell_calls[0]["commands"] == ["rm -f -- /etc/systemd/system/multi-user.target.wants/shop-nginx.service"]
    assert {"service": "shop.target", "enabled": True, "running": True, "restarted": True}.items() <= calls[1].items()
