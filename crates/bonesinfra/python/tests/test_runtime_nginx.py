from pathlib import Path

from bonesinfra.config.context import DeployContext
from bonesinfra.services.linux.nginx import router as nginx_router, site as nginx_site

from .helpers import make_site_request


def _make_ctx(*, domain: str = ""):
    return DeployContext.from_request(make_site_request(domain=domain))


def _noop(*args, **kwargs):
    del args, kwargs


class _Host:
    def __init__(self, certificate):
        self.certificate = certificate
        self.fact_types = []

    def get_fact(self, fact_type, _path):
        self.fact_types.append(fact_type)
        if isinstance(self.certificate, list):
            return self.certificate.pop(0)
        return self.certificate


def test_runtime_nginx_provisions_site_service_without_a_public_domain(monkeypatch):
    ctx = _make_ctx(domain="")
    paths = ctx.paths_dict
    deploy_calls = []

    monkeypatch.setattr(nginx_router, "mkdir", _noop)
    monkeypatch.setattr(nginx_router.service, "render_target", _noop)
    monkeypatch.setattr(nginx_router.service, "register_service", _noop)
    monkeypatch.setattr(nginx_router.files, "link", _noop)
    monkeypatch.setattr(nginx_router.server, "shell", _noop)
    monkeypatch.setattr(nginx_router.systemd, "daemon_reload", _noop)
    monkeypatch.setattr(nginx_router, "install_default_deny_server", _noop)
    monkeypatch.setattr(nginx_router, "validate_config", _noop)
    monkeypatch.setattr(nginx_router, "render", _noop)
    monkeypatch.setattr(
        nginx_router,
        "deploy_router_config",
        lambda *_args, **_kwargs: deploy_calls.append("router"),
    )

    nginx_router.setup(ctx, paths)

    assert deploy_calls == []


def test_deploy_router_config_activates_before_validation_and_reload(monkeypatch):
    ctx = _make_ctx(domain="example.com")
    paths = ctx.paths_dict
    calls = []

    monkeypatch.setattr(nginx_router, "render", lambda *_args, **_kwargs: calls.append("render"))

    def record_link(**kwargs):
        calls.append("link")
        assert kwargs["path"] == paths["nginx_site_enabled"]
        assert kwargs["target"] == paths["nginx_site_available"]
        assert kwargs["force"] is True
        assert kwargs["_sudo"] is True

    monkeypatch.setattr(nginx_router.files, "link", record_link)
    monkeypatch.setattr(nginx_router, "validate_config", lambda *_args: calls.append("validate"))
    monkeypatch.setattr(nginx_router.systemd, "service", lambda **_kwargs: calls.append("reload"))

    nginx_router.deploy_router_config(ctx, paths, ssl_enabled=False, validate=True, reload=True)

    assert calls == ["render", "link", "validate", "reload"]


def test_runtime_nginx_uses_http_router_until_configured_certificate_exists(monkeypatch):
    ctx = _make_ctx(domain="example.com")
    paths = ctx.paths_dict
    deploy_calls = []
    link_calls = []

    monkeypatch.setattr(nginx_router, "mkdir", _noop)
    monkeypatch.setattr(nginx_router.service, "render_target", _noop)
    monkeypatch.setattr(nginx_router.service, "register_service", _noop)
    monkeypatch.setattr(nginx_router.systemd, "daemon_reload", _noop)
    monkeypatch.setattr(nginx_router.files, "link", lambda **kwargs: link_calls.append(kwargs))
    monkeypatch.setattr(nginx_router, "install_default_deny_server", _noop)
    monkeypatch.setattr(nginx_router, "validate_config", _noop)
    monkeypatch.setattr(nginx_router, "render", _noop)
    monkeypatch.setattr(nginx_router.ctx_host, "get", lambda: _Host(None))
    monkeypatch.setattr(
        nginx_router,
        "deploy_router_config",
        lambda *_args, **kwargs: deploy_calls.append(kwargs),
    )

    nginx_router.setup(ctx, paths)

    assert deploy_calls == [{"ssl_enabled": False, "validate": True}]
    assert link_calls == []


def test_runtime_nginx_uses_https_router_when_configured_certificate_exists(monkeypatch):
    ctx = _make_ctx(domain="example.com")
    paths = ctx.paths_dict
    deploy_calls = []
    host = _Host({"link_target": "../../archive/example.com/cert.pem"})

    monkeypatch.setattr(nginx_router, "mkdir", _noop)
    monkeypatch.setattr(nginx_router.service, "render_target", _noop)
    monkeypatch.setattr(nginx_router.service, "register_service", _noop)
    monkeypatch.setattr(nginx_router.systemd, "daemon_reload", _noop)
    monkeypatch.setattr(nginx_router.files, "link", _noop)
    monkeypatch.setattr(nginx_router, "install_default_deny_server", _noop)
    monkeypatch.setattr(nginx_router, "validate_config", _noop)
    monkeypatch.setattr(nginx_router, "render", _noop)
    monkeypatch.setattr(
        nginx_router.ctx_host,
        "get",
        lambda: host,
    )
    monkeypatch.setattr(
        nginx_router,
        "deploy_router_config",
        lambda *_args, **kwargs: deploy_calls.append(kwargs),
    )

    nginx_router.setup(ctx, paths)

    assert deploy_calls == [{"ssl_enabled": True, "validate": True}]
    assert host.fact_types == [nginx_router.Link, nginx_router.Link]


def test_runtime_nginx_uses_http_router_when_certificate_key_is_missing(monkeypatch):
    ctx = _make_ctx(domain="example.com")
    paths = ctx.paths_dict
    deploy_calls = []

    monkeypatch.setattr(nginx_router, "mkdir", _noop)
    monkeypatch.setattr(nginx_router.service, "render_target", _noop)
    monkeypatch.setattr(nginx_router.service, "register_service", _noop)
    monkeypatch.setattr(nginx_router.systemd, "daemon_reload", _noop)
    monkeypatch.setattr(nginx_router.files, "link", _noop)
    monkeypatch.setattr(nginx_router, "install_default_deny_server", _noop)
    monkeypatch.setattr(nginx_router, "validate_config", _noop)
    monkeypatch.setattr(nginx_router, "render", _noop)
    host = _Host([{"mode": 644}, False])
    monkeypatch.setattr(nginx_router.ctx_host, "get", lambda: host)
    monkeypatch.setattr(
        nginx_router,
        "deploy_router_config",
        lambda *_args, **kwargs: deploy_calls.append(kwargs),
    )

    nginx_router.setup(ctx, paths)

    assert deploy_calls == [{"ssl_enabled": False, "validate": True}]


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
    ctx = _make_ctx()
    calls = []

    monkeypatch.setattr(nginx_site.files, "template", _noop)
    monkeypatch.setattr(nginx_site.files, "directory", _noop)
    monkeypatch.setattr(nginx_site.validation.server, "shell", lambda **kwargs: calls.append(kwargs))

    nginx_site.render_static(ctx, paths=ctx.paths_dict, template_src=Path("static-site-nginx.conf.j2"))

    assert calls[0]["_sudo_user"] == "lawsnipe"
