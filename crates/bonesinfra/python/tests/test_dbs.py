from pathlib import Path
from types import SimpleNamespace

import bonesinfra.services.runtime.mongodb as mongodb_mod
import bonesinfra.services.runtime.postgres as postgres_mod
import bonesinfra.services.runtime.valkey as valkey_mod
from bonesinfra.services.runtime import get_service
from bonesinfra.services.runtime.base import RuntimeService


def _ctx(project="atlas-api"):
    return SimpleNamespace(
        app=SimpleNamespace(project_name=project),
        runtime=SimpleNamespace(runtime_group="atlas-api"),
        paths_dict={"shared": "/srv/sites/atlas/shared"},
    )


def _noop(**kwargs):
    pass


def test_all_expected_services_are_registered():
    for name in ("postgres", "mysql", "mariadb", "mongodb", "valkey", "redis"):
        svc = get_service(name)
        assert callable(getattr(svc, "provision", None)), f"{name}: missing provision()"


def test_postgres_installs_postgresql(monkeypatch):
    installed = []
    monkeypatch.setattr(postgres_mod.apt, "packages", lambda **kwargs: installed.append(kwargs))
    monkeypatch.setattr(postgres_mod.server, "script_template", _noop)
    monkeypatch.setattr(postgres_mod.systemd, "service", _noop)

    get_service("postgres").provision(_ctx())

    assert installed[0]["packages"] == ["postgresql"]


def test_valkey_installs_valkey_server(monkeypatch):
    installed = []
    monkeypatch.setattr(valkey_mod.apt, "packages", lambda **kwargs: installed.append(kwargs))
    monkeypatch.setattr(valkey_mod.server, "script_template", _noop)
    monkeypatch.setattr(valkey_mod.systemd, "service", _noop)

    get_service("valkey").provision(_ctx())

    assert installed[0]["packages"] == ["valkey-server"]


def test_valkey_creates_isolated_per_project_instance(monkeypatch):
    template_calls = []
    monkeypatch.setattr(valkey_mod.apt, "packages", _noop)
    monkeypatch.setattr(valkey_mod.server, "script_template", lambda **kwargs: template_calls.append(kwargs))
    monkeypatch.setattr(valkey_mod.systemd, "service", _noop)

    get_service("valkey").provision(_ctx())

    call = template_calls[0]
    assert call["service"] == "valkey"
    assert call["data"] == "/var/lib/valkey/atlas_api"
    assert call["unit"] == "valkey-server"


def test_mongodb_project_account_is_not_a_cluster_admin(monkeypatch):
    template_calls = []
    monkeypatch.setattr(mongodb_mod.server, "shell", _noop)
    monkeypatch.setattr(mongodb_mod.server, "script", _noop)
    monkeypatch.setattr(mongodb_mod.server, "script_template", lambda **kwargs: template_calls.append(kwargs))
    monkeypatch.setattr(mongodb_mod.apt, "packages", _noop)
    monkeypatch.setattr(mongodb_mod.systemd, "service", _noop)

    get_service("mongodb").provision(_ctx())

    call = template_calls[0]
    assert call["project"] == "atlas_api"
    assert call["user"] == "atlas_api_mongodb"
    assert call["env"] == "/srv/sites/atlas/shared/.env"
    assert call["admin_file"] == "/root/.config/bonesinfra/mongodb-admin.env"

    script = Path(call["src"]).read_text()
    assert "updateUser" in script
    assert "roles: [{role: 'readWrite', db: '$PROJECT'}]" in script


def test_database_identifier_rejects_unsafe_project_names():
    assert RuntimeService._db_identifier("atlas-api") == "atlas_api"
    try:
        RuntimeService._db_identifier("atlas;drop")
    except ValueError:
        pass
    else:
        raise AssertionError("unsafe database identifier was accepted")
