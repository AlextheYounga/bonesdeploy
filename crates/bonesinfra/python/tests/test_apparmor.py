"""Regression: app-profile.j2 reads shared template context ({{ paths.releases }},
{{ project_name }}, ...), so render_profile must forward template_data or the
profile fails at provision time with an undefined variable."""

import types

import jinja2

from bonesinfra.config.context import DeployContext
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.frameworks.sveltekit.runtime import TEMPLATES as SVELTEKIT_TEMPLATES
from bonesinfra.services.linux.apparmor import app as apparmor_app

from .helpers import make_site_request


def _ctx() -> DeployContext:
    return DeployContext.from_request(make_site_request())


def test_render_profile_forwards_template_context(tmp_path, monkeypatch):
    ctx = _ctx()
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


def test_sveltekit_profile_permits_reading_the_shared_environment(tmp_path, monkeypatch):
    ctx = _ctx()
    seen = {}

    def _capture_render(_name, _src, _dest, **data):
        seen["data"] = data
        return types.SimpleNamespace(changes=[])

    monkeypatch.setattr(apparmor_app, "render", _capture_render)
    monkeypatch.setattr(apparmor_app.server, "shell", lambda **_kw: types.SimpleNamespace(changes=[]))

    paths = ctx.paths_dict
    apparmor_app.render_profile(
        ctx,
        paths=paths,
        runtime="sveltekit",
        template_src=SVELTEKIT_TEMPLATES / "app-profile.j2",
        apparmor_exec_paths=["/usr/bin/node"],
        apparmor_writable_paths=[],
    )

    rendered = (
        jinja2.Environment(autoescape=True, loader=jinja2.FileSystemLoader(str(SVELTEKIT_TEMPLATES)))
        .get_template("app-profile.j2")
        .render(seen["data"])
    )
    assert "/srv/sites/lawsnipe/shared/.env r," in rendered
