"""The deploy sudo boundary permits only two exact BonesRemote argument forms."""

import re

import pytest
from jinja2 import Environment, StrictUndefined

from bonesinfra.cli.commands.server import sudoers

from . import helpers

SUDOERS_TEMPLATE = helpers.SRC_DIR / "bonesinfra/assets/sudoers/bonesdeploy.j2"
DEPLOY_USER = "git"
BONESREMOTE_PATH = "/usr/local/bin/bonesremote"
SUDOERS_PATH = "/etc/sudoers.d/bonesdeploy"

ALLOWED_RULE = (
    f"{DEPLOY_USER} ALL=(root) NOPASSWD: {BONESREMOTE_PATH} ^config sync --site [a-z0-9-]+$, "
    f"{BONESREMOTE_PATH} ^deploy --site [a-z0-9-]+$"
)


def _render(**data):
    # Mirror pyinfra's rendering environment: plain text, strict variables.
    env = Environment(autoescape=False, undefined=StrictUndefined, keep_trailing_newline=True)  # noqa: S701
    return env.from_string(helpers.read(SUDOERS_TEMPLATE)).render(**data)


def _rendered_rule():
    rendered = _render(deploy_user=DEPLOY_USER, bonesremote_path=BONESREMOTE_PATH)
    rules = [line for line in rendered.splitlines() if line.startswith(f"{DEPLOY_USER} ")]
    assert len(rules) == 1
    return rendered, rules[0]


def _argument_patterns(rule):
    # Sudo joins the command line arguments with single spaces and matches the
    # anchored POSIX ERE against that string.
    command_spec = rule.split(" NOPASSWD: ", 1)[1]
    patterns = []
    for command in command_spec.split(", "):
        path, pattern = command.split(" ", 1)
        assert path == BONESREMOTE_PATH
        assert pattern.startswith("^") and pattern.endswith("$")
        patterns.append(re.compile(pattern))
    return patterns


def test_sudoers_pins_the_direct_bonesremote_allowlist():
    rendered, rule = _rendered_rule()

    assert rule == ALLOWED_RULE
    assert "--config-stdin" not in rendered
    assert "wrapper" not in rendered


@pytest.mark.parametrize(
    "argv",
    [
        ["config", "sync", "--site", "demo"],
        ["config", "sync", "--site", "atlas-2"],
        ["deploy", "--site", "demo"],
        ["deploy", "--site", "atlas-2"],
    ],
)
def test_allowed_argument_forms_are_permitted(argv):
    _, rule = _rendered_rule()

    command_line = " ".join(argv)
    assert any(pattern.fullmatch(command_line) for pattern in _argument_patterns(rule))


@pytest.mark.parametrize(
    "argv",
    [
        [],
        ["config", "sync"],
        ["deploy"],
        ["config", "sync", "--site"],
        ["deploy", "--site"],
        ["deploy", "demo"],
        ["deploy", "--site=demo"],
        ["config", "sync", "--site", "demo", "extra"],
        ["deploy", "--site", "demo", "--config-stdin"],
        ["config", "sync", "--config-stdin", "--site", "demo"],
        ["config", "sync", "--config-stdin"],
        ["--config-stdin"],
        ["status", "demo"],
        ["sync", "config", "--site", "demo"],
        ["config", "sync", "--site", "Demo"],
        ["config", "sync", "--site", "demo_x"],
        ["config", "sync", "--site", "demo/x"],
        ["config", "sync", "--site", "demo.x"],
        ["deploy", "--site", "demo site"],
        ["deploy", "--site", "demo;reboot"],
        ["deploy", "--site", "$(reboot)"],
        ["deploy", "--site", "`reboot`"],
        ["deploy", "--site", ""],
        ["deploy", "--site", "demo\n"],
    ],
)
def test_every_other_argument_form_is_denied(argv):
    _, rule = _rendered_rule()

    command_line = " ".join(argv)
    assert not any(pattern.fullmatch(command_line) for pattern in _argument_patterns(rule))


def test_install_renders_the_sudoers_for_the_deploy_identity(monkeypatch):
    calls = []
    monkeypatch.setattr(sudoers, "render", lambda *args, **kwargs: calls.append(("render", args, kwargs)))
    monkeypatch.setattr(sudoers.server, "shell", lambda **kwargs: calls.append(("shell", kwargs)))

    sudoers.install()

    assert [kind for kind, *_rest in calls] == ["render", "shell"]

    render_call = calls[0]
    assert render_call[1][0] == "Install BonesDeploy sudoers drop-in"
    assert render_call[1][1] == SUDOERS_TEMPLATE
    assert render_call[1][2] == SUDOERS_PATH
    render_kwargs = render_call[2]
    assert render_kwargs["user"] == render_kwargs["group"] == "root"
    assert render_kwargs["mode"] == "0440"
    assert render_kwargs["deploy_user"] == DEPLOY_USER
    assert render_kwargs["bonesremote_path"] == BONESREMOTE_PATH

    shell_kwargs = calls[1][1]
    assert any("visudo -c -f /etc/sudoers.d/bonesdeploy" in command for command in shell_kwargs["commands"])
