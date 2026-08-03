"""Regression: app-profile.j2 reads {{ paths.releases }}, so render_profile must
forward a resolvable `paths` object or the profile fails at provision time with
`'paths' is undefined`."""

import types
from pathlib import Path

from bonesinfra.config.context import DeployContext
from bonesinfra.config.paths import DeploymentPaths
from bonesinfra.services.linux.apparmor import app as apparmor_app


def _ctx(tmp_path: Path) -> DeployContext:
    config_path = tmp_path / "bones.toml"
    config_path.write_text(
        """[app]
project_name = "lawsnipe"

[app.server]
host = "example.com"
port = "22"
"""
    )
    return DeployContext.from_files(str(config_path))


def test_render_profile_forwards_paths_to_template(tmp_path, monkeypatch):
    ctx = _ctx(tmp_path)
    seen = {}

    def _capture_render(name, src, dest, **data):
        seen["name"] = name
        seen["data"] = data
        return types.SimpleNamespace(changes=[])

    monkeypatch.setattr(apparmor_app, "render", _capture_render)
    monkeypatch.setattr(apparmor_app.server, "shell", lambda **_kw: types.SimpleNamespace(changes=[]))

    deployments = DeploymentPaths.new("lawsnipe", "/home/git/lawsnipe.git", "/srv/sites/lawsnipe")
    apparmor_app.render_profile(
        ctx,
        paths=deployments,
        runtime="next",
        apparmor_exec_paths=["/usr/bin/node"],
        apparmor_writable_paths=[str(deployments.shared)],
    )

    assert seen["name"] == "Deploy next AppArmor profile"
    assert seen["data"]["paths"] is deployments
    assert seen["data"]["apparmor_runtime"] == "next"
