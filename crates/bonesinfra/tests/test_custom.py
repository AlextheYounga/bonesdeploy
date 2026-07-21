"""``.bones/custom.py`` extension hook contract."""

from __future__ import annotations

import types
from pathlib import Path

import pytest

from bonesinfra.deploys.helpers import plan as helpers_plan
from bonesinfra.deploys.runtime import plan as runtime_plan
from bonesinfra.deploys.setup import plan as setup_plan
from bonesinfra.deploys.ssl import plan as ssl_plan
from bonesinfra.domain import custom as custom_mod
from bonesinfra.domain.context import DeployContext


def _write_config(tmp: Path) -> Path:
    path = tmp / "bones.toml"
    path.write_text(
        """[app]
project_name = "lawsnipe"
[app.server]
host = "example.com"
ssh_user = "root"
port = 2222
[app.dns]
domain = "example.com"
email = "ops@example.com"
ssl_enabled = true
"""
    )
    return path


def _ctx(tmp: Path) -> DeployContext:
    return DeployContext.from_files(str(_write_config(tmp)))


def _custom(tmp: Path, body: str) -> Path:
    path = tmp / "custom.py"
    path.write_text(body)
    return path


def _rec(marker: str, record: list[str]) -> types.SimpleNamespace:
    def _f(*args, **kwargs):
        del args, kwargs
        record.append(marker)

    return _f


def test_load_custom_module_returns_none_when_missing(tmp_path):
    _write_config(tmp_path)
    assert custom_mod.load_custom_module(str(tmp_path / "bones.toml")) is None


def test_call_hook_noop_when_module_missing(tmp_path):
    custom_mod.call_hook(None, "after_setup", _ctx(tmp_path))


def test_load_custom_module_loads_hooks(tmp_path):
    _custom(
        tmp_path,
        "def after_setup(ctx):\n    return ctx\n\ndef after_runtime(ctx):\n    pass\n",
    )
    mod = custom_mod.load_custom_module(str(tmp_path / "bones.toml"))
    assert mod is not None
    assert callable(mod.after_setup)
    assert callable(mod.after_runtime)


def test_syntax_error_exits_before_ssh(tmp_path):
    _custom(tmp_path, "def after_setup(ctx:\n    pass\n")
    _write_config(tmp_path)
    with pytest.raises(SystemExit) as exc:
        custom_mod.load_custom_module(str(tmp_path / "bones.toml"))
    assert exc.value.code == 3


def test_non_callable_hook_exits_before_ssh(tmp_path, capsys):
    _custom(tmp_path, "after_setup = 5\n")
    _write_config(tmp_path)
    with pytest.raises(SystemExit) as exc:
        custom_mod.load_custom_module(str(tmp_path / "bones.toml"))
    assert exc.value.code == 3
    err = capsys.readouterr().err
    assert "after_setup" in err
    assert "custom.py" in err


def test_custom_path_resolves_next_to_config(tmp_path):
    config = tmp_path / "nested" / "bones.toml"
    config.parent.mkdir()
    config.write_text("[app]\nproject_name = 'x'\n")
    assert custom_mod.custom_path_for(str(config)) == (tmp_path / "nested" / "custom.py").resolve()


def _stub_setup_plan(monkeypatch, record: list[str]) -> None:
    monkeypatch.setattr(setup_plan, "BASE_SYSTEM_PACKAGES", [])
    monkeypatch.setattr(setup_plan, "SUPPLEMENTARY_PACKAGES", [])
    monkeypatch.setattr(setup_plan, "packages", types.SimpleNamespace(install_system=_rec("install_system", record)))
    monkeypatch.setattr(
        setup_plan, "kernel_hardening", types.SimpleNamespace(configure=_rec("kernel_hardening", record))
    )
    monkeypatch.setattr(
        setup_plan,
        "users",
        types.SimpleNamespace(
            install_rust=_rec("install_rust", record),
            ensure_users_and_groups=_rec("ensure_users_and_groups", record),
            install_authorized_key=_rec("install_authorized_key", record),
        ),
    )
    monkeypatch.setattr(
        setup_plan,
        "image_store",
        types.SimpleNamespace(
            ensure_shared_store=_rec("ensure_shared_store", record), seed_base_image=_rec("seed_base_image", record)
        ),
    )
    monkeypatch.setattr(
        setup_plan, "directories", types.SimpleNamespace(setup_repo_and_project=_rec("directories", record))
    )
    monkeypatch.setattr(setup_plan, "placeholder", types.SimpleNamespace(seed=_rec("placeholder", record)))
    monkeypatch.setattr(setup_plan, "firewall", types.SimpleNamespace(configure=_rec("firewall", record)))
    monkeypatch.setattr(setup_plan, "fail2ban", types.SimpleNamespace(configure=_rec("fail2ban", record)))
    monkeypatch.setattr(
        setup_plan, "unattended_upgrades", types.SimpleNamespace(configure=_rec("unattended_upgrades", record))
    )
    monkeypatch.setattr(setup_plan, "bonesremote", types.SimpleNamespace(install=_rec("bonesremote", record)))
    monkeypatch.setattr(setup_plan, "sudoers", types.SimpleNamespace(install=_rec("sudoers", record)))


def test_after_setup_runs_last_and_receives_ctx(tmp_path, monkeypatch):
    record: list[str] = []
    _stub_setup_plan(monkeypatch, record)

    ctx = _ctx(tmp_path)
    received: list[DeployContext] = []
    _custom(
        tmp_path,
        "def after_setup(ctx):\n"
        "    globals()['_record'].append('after_setup')\n"
        "    globals()['_received'].append(ctx)\n",
    )
    mod = custom_mod.load_custom_module(str(tmp_path / "bones.toml"))
    monkeypatch.setattr(mod, "_record", record, raising=False)
    monkeypatch.setattr(mod, "_received", received, raising=False)

    setup_plan.deploy_setup(ctx, mod)

    assert record[-1] == "after_setup"
    assert received == [ctx]


def test_only_invoked_phase_hook_runs(tmp_path, monkeypatch):
    record: list[str] = []
    monkeypatch.setattr(runtime_plan, "packages", types.SimpleNamespace(install_apt=_rec("apt", record)))
    monkeypatch.setattr(runtime_plan, "apparmor", types.SimpleNamespace(setup=_rec("apparmor", record)))
    monkeypatch.setattr(
        runtime_plan,
        "nginx",
        types.SimpleNamespace(setup=_rec("nginx.setup", record), start_services=_rec("nginx.start", record)),
    )
    monkeypatch.setattr(runtime_plan, "template_runtime", types.SimpleNamespace(load=_rec("template_runtime", record)))
    monkeypatch.setattr(runtime_plan, "get_runtime", _rec("get_runtime", record))

    _custom(
        tmp_path,
        "def after_setup(ctx):\n    raise AssertionError('wrong phase')\n"
        "def after_runtime(ctx):\n    globals()['_record'].append('after_runtime')\n"
        "def after_ssl(ctx):\n    raise AssertionError('wrong phase')\n",
    )
    mod = custom_mod.load_custom_module(str(tmp_path / "bones.toml"))
    monkeypatch.setattr(mod, "_record", record, raising=False)

    runtime_plan.deploy_runtime(_ctx(tmp_path), mod)

    assert record[-1] == "after_runtime"


def test_missing_hook_is_noop_in_ssl_plan(tmp_path, monkeypatch):
    record: list[str] = []
    monkeypatch.setattr(ssl_plan, "mkdir", _rec("mkdir", record))
    monkeypatch.setattr(
        ssl_plan, "nginx_safety", types.SimpleNamespace(install_default_deny_server=_rec("default_deny", record))
    )
    monkeypatch.setattr(ssl_plan, "_render_router_config", _rec("render", record))
    monkeypatch.setattr(ssl_plan, "obtain_certificate", _rec("obtain", record))

    _custom(tmp_path, "# no hooks defined\n")
    mod = custom_mod.load_custom_module(str(tmp_path / "bones.toml"))
    ssl_plan.deploy_ssl(_ctx(tmp_path), mod)
    assert "obtain" in record


def test_helpers_plan_runs_no_hook(tmp_path, monkeypatch):
    record: list[str] = []
    monkeypatch.setattr(
        helpers_plan,
        "packages",
        types.SimpleNamespace(
            install_helper_apt_packages=_rec("apt", record), install_debian_command_aliases=_rec("aliases", record)
        ),
    )
    monkeypatch.setattr(helpers_plan, "starship", types.SimpleNamespace(install=_rec("starship", record)))
    monkeypatch.setattr(helpers_plan, "neovim", types.SimpleNamespace(install=_rec("neovim", record)))
    monkeypatch.setattr(helpers_plan, "rainfrog", types.SimpleNamespace(install=_rec("rainfrog", record)))

    _custom(
        tmp_path,
        "def after_setup(ctx):\n    raise AssertionError('helpers must not run setup hook')\n",
    )
    mod = custom_mod.load_custom_module(str(tmp_path / "bones.toml"))
    helpers_plan.deploy_helpers(_ctx(tmp_path), mod)
    assert "rainfrog" in record
