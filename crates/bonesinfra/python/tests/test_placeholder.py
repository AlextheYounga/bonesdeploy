from bonesinfra.cli.commands.site import placeholder
from bonesinfra.config.context import DeployContext

from .helpers import make_site_request


def _ctx() -> DeployContext:
    return DeployContext.from_request(make_site_request())


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
    ctx = _ctx()
    commands = _seed(ctx, monkeypatch)

    current = ctx.paths_dict["current"]
    placeholder_release = ctx.paths_dict["placeholder_release"]
    assert commands == [f"test -e {current} -o -L {current} || ln -s {placeholder_release} {current}"]
