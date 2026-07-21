from contextlib import contextmanager
from pathlib import Path
from tempfile import TemporaryDirectory

import pyinfra.connectors.ssh as pyinfra_ssh

from bonesinfra.domain.context import DeployContext
from bonesinfra.infra import pyinfra_runner

sentinel_key = object()


@contextmanager
def _noop_activity(_message):
    yield


def _noop_print_target(*args, **kwargs):
    del args, kwargs


def _noop_run_ops(state):
    del state


def _noop_get_private_key(*args, **kwargs):
    del args, kwargs
    return sentinel_key


def _noop_deploy(*args, **kwargs):
    del args, kwargs


def test_run_passes_ssh_auth_through_inventory(monkeypatch):
    with TemporaryDirectory() as tmp:
        config_path = Path(tmp) / "bones.toml"
        config_path.write_text(
            """[app]
project_name = "lawsnipe"
[app.server]
host = "example.com"
ssh_user = "root"
port = 2222
"""
        )

        ctx = DeployContext.from_files(str(config_path))

    seen = {}

    monkeypatch.setattr(pyinfra_runner, "setup_output", lambda: None)
    monkeypatch.setattr(pyinfra_runner, "print_banner", lambda: None)
    monkeypatch.setattr(pyinfra_runner, "print_target", _noop_print_target)
    monkeypatch.setattr(pyinfra_runner, "print_connected", lambda: None)
    monkeypatch.setattr(pyinfra_runner, "print_done", lambda success: seen.setdefault("done", success))
    monkeypatch.setattr(pyinfra_runner, "stop_live_output", lambda: None)
    monkeypatch.setattr(pyinfra_runner, "activity", _noop_activity)
    monkeypatch.setattr(pyinfra_runner, "run_ops", _noop_run_ops)
    monkeypatch.setattr(pyinfra_ssh, "get_private_key", _noop_get_private_key)

    def fake_connect_all(state):
        host = next(iter(state.inventory))
        seen["kwargs"] = host.connector.make_paramiko_kwargs()

    monkeypatch.setattr(pyinfra_runner, "connect_all", fake_connect_all)

    pyinfra_runner.run(ctx=ctx, config_path=str(config_path), ssh_key="~/.ssh/id_ed25519", deploy=_noop_deploy)

    assert seen["kwargs"]["username"] == "root"
    assert seen["kwargs"]["port"] == 2222
    assert seen["kwargs"]["pkey"] is sentinel_key
    assert seen["kwargs"]["allow_agent"] is False
    assert seen["kwargs"]["look_for_keys"] is False
    assert seen["done"] is True
