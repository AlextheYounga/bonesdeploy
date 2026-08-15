"""Regression: app-profile.j2 reads shared template context ({{ paths.releases }},
{{ project_name }}, ...), so render_profile must forward template_data or the
profile fails at provision time with an undefined variable."""

import types
from pathlib import Path

import jinja2

from bonesinfra.config.context import DeployContext
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.services.linux.apparmor import app as apparmor_app


def _ctx(tmp_path: Path) -> DeployContext:
    config_path = tmp_path / ".env"
    config_path.write_text(
        """PROJECT_NAME=lawsnipe
HOST=example.com
PORT=22
"""
    )
    return DeployContext.from_files(str(config_path))


def test_render_profile_forwards_template_context(tmp_path, monkeypatch):
    ctx = _ctx(tmp_path)
    seen = {}

    def _capture_render(name, src, dest, **data):
        seen["name"] = name
        seen["data"] = data
        return types.SimpleNamespace(changes=[])

    monkeypatch.setattr(apparmor_app, "render", _capture_render)
    monkeypatch.setattr(apparmor_app.server, "shell", lambda **_kw: types.SimpleNamespace(changes=[]))

    paths = ctx.paths_dict
    apparmor_app.render_profile(
        ctx,
        paths=paths,
        runtime="next",
        template_src=ASSETS_DIR / "apparmor/app-profile.j2",
        apparmor_exec_paths=["/usr/bin/node"],
        apparmor_writable_paths=[paths["shared"]],
    )

    assert seen["name"] == "Deploy next AppArmor profile"
    assert seen["data"]["project_name"] == "lawsnipe"
    assert seen["data"]["paths"] is paths
    assert seen["data"]["apparmor_runtime"] == "next"

    rendered = (
        jinja2.Environment(autoescape=True, loader=jinja2.FileSystemLoader(str(ASSETS_DIR)))
        .get_template("apparmor/app-profile.j2")
        .render(seen["data"])
    )
    assert "/srv/sites/lawsnipe/releases/*/** r," in rendered
    assert "/var/log/bonesdeploy/lawsnipe/ rw," in rendered
