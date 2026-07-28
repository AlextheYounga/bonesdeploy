import pytest

from bonesinfra.config.context import DeployContext
from bonesinfra.services.linux.nginx import router as nginx_router
from bonesinfra.services.linux.nginx import site as nginx_site


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

    monkeypatch.setattr(nginx_router, "mkdir", _noop)
    monkeypatch.setattr(nginx_router.service, "render_target", _noop)
    monkeypatch.setattr(nginx_router.service, "register_service", _noop)
    monkeypatch.setattr(nginx_router.files, "link", _noop)
    monkeypatch.setattr(nginx_router.server, "shell", _noop)
    monkeypatch.setattr(nginx_router.systemd, "daemon_reload", _noop)
    monkeypatch.setattr(nginx_router, "install_default_deny_server", _noop)
    monkeypatch.setattr(nginx_router, "validate_config", _noop)

    def fake_render(*args, **kwargs):
        calls.append((args, kwargs))

    monkeypatch.setattr(nginx_router, "render", fake_render)

    nginx_router.setup(ctx, paths)

    router_call = next(call for _, call in calls if "nginx_server_name" in call)
    assert router_call["nginx_server_name"] == "preview.example.com"
    assert router_call["preview_domain"] == "preview.example.com"


def test_runtime_nginx_requires_a_real_name(tmp_path, monkeypatch):
    ctx = _make_ctx(tmp_path, domain="", preview_domain="")
    paths = ctx.paths_dict

    monkeypatch.setattr(nginx_router, "mkdir", _noop)
    monkeypatch.setattr(nginx_router.service, "render_target", _noop)
    monkeypatch.setattr(nginx_router.service, "register_service", _noop)
    monkeypatch.setattr(nginx_router.files, "link", _noop)
    monkeypatch.setattr(nginx_router.server, "shell", _noop)
    monkeypatch.setattr(nginx_router.systemd, "daemon_reload", _noop)
    monkeypatch.setattr(nginx_router, "install_default_deny_server", _noop)
    monkeypatch.setattr(nginx_router, "validate_config", _noop)
    monkeypatch.setattr(nginx_router, "render", _noop)

    with pytest.raises(ValueError, match="domain or preview_domain"):
        nginx_router.setup(ctx, paths)


def test_runtime_nginx_migrates_site_service_to_target(monkeypatch):
    calls = []
    shell_calls = []
    ctx = type("Context", (), {"app": type("App", (), {"project_name": "shop"})()})()
    paths = {
        "systemd_site_nginx_service": "/etc/systemd/system/shop-nginx.service",
        "systemd_site_target": "/etc/systemd/system/shop.target",
    }
    monkeypatch.setattr(nginx_router.systemd, "service", lambda **kwargs: calls.append(kwargs))
    monkeypatch.setattr(nginx_router.server, "shell", lambda **kwargs: shell_calls.append(kwargs))

    nginx_router.start_services(ctx, paths)

    assert shell_calls[0]["commands"] == ["rm -f -- /etc/systemd/system/multi-user.target.wants/shop-nginx.service"]
    assert {"service": "shop.target", "enabled": True, "running": True, "restarted": True}.items() <= calls[1].items()


def test_static_nginx_validation_runs_as_runtime_user(tmp_path, monkeypatch):
    ctx = _make_ctx(tmp_path)
    calls = []

    monkeypatch.setattr(nginx_site.files, "template", _noop)
    monkeypatch.setattr(nginx_site.files, "directory", _noop)
    monkeypatch.setattr(nginx_site.validation.server, "shell", lambda **kwargs: calls.append(kwargs))

    nginx_site.render_static(ctx, paths=ctx.paths_dict)

    assert calls[0]["_sudo_user"] == "lawsnipe"
