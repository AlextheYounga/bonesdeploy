from pathlib import Path

from bonesinfra.cli.commands.setup import placeholder
from bonesinfra.config.context import DeployContext


def _ctx(tmp: Path) -> DeployContext:
    config = tmp / "bones.toml"
    config.write_text(
        """[app]
project_name = "lawsnipe"
[app.server]
host = "example.com"
ssh_user = "root"
port = 2222
"""
    )
    return DeployContext.from_files(str(config))


def _seed(ctx, monkeypatch):
    calls = []

    def _noop_render(*_args, **_kwargs):
        return None

    def _record_shell(*_args, **kwargs):
        calls.append(kwargs["commands"][0])

    monkeypatch.setattr(placeholder, "render", _noop_render)
    monkeypatch.setattr(placeholder.server, "shell", _record_shell)
    placeholder.seed(ctx, ctx.paths_dict)
    return calls


def test_seed_never_replaces_an_existing_current_release(tmp_path, monkeypatch):
    ctx = _ctx(tmp_path)
    commands = _seed(ctx, monkeypatch)

    current = ctx.paths_dict["current"]
    placeholder_release = ctx.paths_dict["placeholder_release"]
    assert commands == [f"test -e {current} -o -L {current} || ln -s {placeholder_release} {current}"]
