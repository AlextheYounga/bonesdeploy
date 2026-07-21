from pathlib import Path
from types import SimpleNamespace

from bonesinfra.deploys.dbs import services


def _ctx(service_names):
    return SimpleNamespace(
        app=SimpleNamespace(project_name="atlas-api"),
        runtime=SimpleNamespace(runtime_group="atlas-api"),
        dbs=SimpleNamespace(services=tuple(service_names)),
        paths_dict={"shared": "/srv/sites/atlas/shared"},
    )


def test_selected_services_only_schedule_selected_packages(monkeypatch):
    installed = []
    commands = []
    template_calls = []
    monkeypatch.setattr(services.apt, "packages", lambda **kwargs: installed.append(kwargs))
    monkeypatch.setattr(services.server, "shell", lambda **kwargs: commands.append(kwargs))
    monkeypatch.setattr(services.server, "script_template", lambda **kwargs: template_calls.append(kwargs))
    monkeypatch.setattr(services.systemd, "service", lambda **kwargs: commands.append(kwargs))

    services.provision(_ctx(["postgres", "valkey"]))

    assert installed[0]["packages"] == ["postgresql", "valkey-server"]
    assert all("mysql" not in str(command) for command in commands)

    valkey_call = next(call for call in template_calls if call.get("service") == "valkey")
    assert valkey_call["service"] == "valkey"
    assert valkey_call["data"] == "/var/lib/valkey/atlas_api"
    assert valkey_call["unit"] == "valkey-server"


def test_mongodb_project_account_is_not_a_cluster_admin(monkeypatch):
    template_calls = []

    def _noop(**kwargs):
        pass

    monkeypatch.setattr(services.server, "shell", _noop)
    monkeypatch.setattr(services.server, "script_template", lambda **kwargs: template_calls.append(kwargs))
    monkeypatch.setattr(services.systemd, "service", _noop)

    services._mongodb("atlas_api", "/srv/sites/atlas/shared/.env")

    call = template_calls[0]
    assert call["project"] == "atlas_api"
    assert call["user"] == "atlas_api_mongodb"
    assert call["env"] == "/srv/sites/atlas/shared/.env"
    assert call["admin_file"] == "/root/.config/bonesinfra/mongodb-admin.env"

    script = Path(call["src"]).read_text()
    assert "updateUser" in script
    assert "roles: [{role: 'readWrite', db: '$PROJECT'}]" in script


def test_database_identifier_rejects_unsafe_project_names():
    assert services._database_identifier("atlas-api") == "atlas_api"
    try:
        services._database_identifier("atlas;drop")
    except ValueError:
        pass
    else:
        raise AssertionError("unsafe database identifier was accepted")
