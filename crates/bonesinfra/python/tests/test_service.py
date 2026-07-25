from types import SimpleNamespace

import pytest

from bonesinfra.runtimes.common import service


def test_register_service_uses_exact_project_derived_unit(monkeypatch):
    calls = []
    monkeypatch.setattr(service.server, "shell", lambda **kwargs: calls.append(kwargs))
    ctx = SimpleNamespace(app=SimpleNamespace(project_name="shop"))
    paths = {"systemd_site_target_requires": "/etc/systemd/system/shop.target.requires"}

    service.register_service(ctx, paths=paths, name="nginx")

    command = calls[0]["commands"][0]
    assert "ln -sfn -- /etc/systemd/system/shop-nginx.service" in command
    assert "/etc/systemd/system/shop.target.requires/shop-nginx.service" in command
    assert "shop-admin" not in command


def test_register_service_rejects_arbitrary_unit_suffix(monkeypatch):
    def ignore_link(*_args, **_kwargs):
        return None

    monkeypatch.setattr(service.server, "shell", ignore_link)
    ctx = SimpleNamespace(app=SimpleNamespace(project_name="shop"))
    paths = {"systemd_site_target_requires": "/etc/systemd/system/shop.target.requires"}

    with pytest.raises(ValueError, match="invalid site service name"):
        service.register_service(ctx, paths=paths, name="admin.service")

    with pytest.raises(ValueError, match="invalid site service name"):
        service.register_service(ctx, paths=paths, name="-admin")


def test_render_target_reconciles_stale_memberships(monkeypatch):
    def ignore_template(*_args, **_kwargs):
        return None

    def empty_template_data(*_args, **_kwargs):
        return {}

    shell_calls = []
    monkeypatch.setattr(service.files, "template", ignore_template)
    monkeypatch.setattr(service.server, "shell", lambda **kwargs: shell_calls.append(kwargs))
    monkeypatch.setattr(service, "template_data", empty_template_data)
    ctx = SimpleNamespace(app=SimpleNamespace(project_name="shop"))
    paths = {
        "systemd_site_target": "/etc/systemd/system/shop.target",
        "systemd_site_target_requires": "/etc/systemd/system/shop.target.requires",
    }

    service.render_target(ctx, paths=paths)

    assert shell_calls[0]["commands"] == [
        "rm -rf -- /etc/systemd/system/shop.target.requires; "
        "install -d -o root -g root -m 0755 -- /etc/systemd/system/shop.target.requires",
    ]


def test_target_membership_reconciliation_recreates_all_registered_services(monkeypatch):
    def ignore_template(*_args, **_kwargs):
        return None

    def empty_template_data(*_args, **_kwargs):
        return {}

    calls = []
    monkeypatch.setattr(service.files, "template", ignore_template)
    monkeypatch.setattr(service.server, "shell", lambda **kwargs: calls.append(kwargs))
    monkeypatch.setattr(service, "template_data", empty_template_data)
    ctx = SimpleNamespace(app=SimpleNamespace(project_name="nexttest"))
    paths = {
        "systemd_site_target": "/etc/systemd/system/nexttest.target",
        "systemd_site_target_requires": "/etc/systemd/system/nexttest.target.requires",
    }

    service.render_target(ctx, paths=paths)
    service.register_service(ctx, paths=paths, name="nginx")
    service.register_service(ctx, paths=paths, name="next")

    commands = [call["commands"][0] for call in calls]
    assert commands[0].startswith("rm -rf -- /etc/systemd/system/nexttest.target.requires")
    assert commands[1].endswith("nexttest.target.requires/nexttest-nginx.service")
    assert commands[2].endswith("nexttest.target.requires/nexttest-next.service")


def test_enable_and_start_removes_legacy_direct_service_enablement(monkeypatch):
    shell_calls = []
    systemd_calls = []
    monkeypatch.setattr(service.server, "shell", lambda **kwargs: shell_calls.append(kwargs))
    monkeypatch.setattr(service.systemd, "service", lambda **kwargs: systemd_calls.append(kwargs))
    ctx = SimpleNamespace(app=SimpleNamespace(project_name="shop"))

    service.enable_and_start(ctx, "next")

    assert shell_calls[0]["commands"] == ["rm -f -- /etc/systemd/system/multi-user.target.wants/shop-next.service"]
    assert {"service": "shop-next.service", "running": True}.items() <= systemd_calls[0].items()
    assert "enabled" not in systemd_calls[0]
