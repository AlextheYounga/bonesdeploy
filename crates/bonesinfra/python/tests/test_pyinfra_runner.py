from contextlib import contextmanager
from pathlib import Path
from tempfile import TemporaryDirectory

import pyinfra.connectors.ssh as pyinfra_ssh

from bonesinfra.config.context import DeployContext
from bonesinfra.pyinfra import runner

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

    monkeypatch.setattr(runner, "setup_output", lambda: None)
    monkeypatch.setattr(runner, "print_banner", lambda: None)
    monkeypatch.setattr(runner, "print_target", _noop_print_target)
    monkeypatch.setattr(runner, "print_connected", lambda: None)
    monkeypatch.setattr(runner, "print_done", lambda success: seen.setdefault("done", success))
    monkeypatch.setattr(runner, "stop_live_output", lambda: None)
    monkeypatch.setattr(runner, "activity", _noop_activity)
    monkeypatch.setattr(runner, "run_ops", _noop_run_ops)
    monkeypatch.setattr(pyinfra_ssh, "get_private_key", _noop_get_private_key)

    def fake_connect_all(state):
        host = next(iter(state.inventory))
        seen["kwargs"] = host.connector.make_paramiko_kwargs()

    monkeypatch.setattr(runner, "connect_all", fake_connect_all)

    runner.run(ctx=ctx, config_path=str(config_path), ssh_key="~/.ssh/id_ed25519", deploy=_noop_deploy)

    assert seen["kwargs"]["username"] == "root"
    assert seen["kwargs"]["port"] == 2222
    assert seen["kwargs"]["pkey"] is sentinel_key
    assert seen["kwargs"]["allow_agent"] is False
    assert seen["kwargs"]["look_for_keys"] is False
    assert seen["done"] is True


def test_run_can_override_ssh_user_for_update_patches(monkeypatch, tmp_path):
    config_path = tmp_path / "bones.toml"
    config_path.write_text(
        """[app]
project_name = "lawsnipe"
[app.server]
host = "example.com"
ssh_user = "deploy"
port = 2222
"""
    )
    ctx = DeployContext.from_files(str(config_path))
    seen = {}

    monkeypatch.setattr(runner, "load_custom_module", lambda _path: None)
    monkeypatch.setattr(
        runner,
        "connect_all",
        lambda state: seen.update(user=next(iter(state.inventory)).data.ssh_user),
    )
    monkeypatch.setattr(runner, "run_ops", _noop_run_ops)

    runner.run(
        ctx=ctx,
        config_path=str(config_path),
        deploy=_noop_deploy,
        ssh_user_override="root",
        quiet=True,
    )

    assert seen["user"] == "root"
