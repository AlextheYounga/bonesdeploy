"""Regression: app-profile.j2 reads shared template context ({{ paths.releases }},
{{ project_name }}, ...), so render_profile must forward template_data or the
profile fails at provision time with an undefined variable."""

import types

import jinja2
import pytest

from bonesinfra.config.context import DeployContext
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.frameworks.django.runtime import TEMPLATES as DJANGO_TEMPLATES
from bonesinfra.frameworks.next.runtime import TEMPLATES as NEXT_TEMPLATES
from bonesinfra.frameworks.nuxt.runtime import TEMPLATES as NUXT_TEMPLATES
from bonesinfra.frameworks.rails.runtime import (
    TEMPLATES as RAILS_TEMPLATES,
    apparmor_exec_paths as rails_apparmor_exec_paths,
)
from bonesinfra.frameworks.sveltekit.runtime import TEMPLATES as SVELTEKIT_TEMPLATES
from bonesinfra.services.languages.mise import MiseRuntimeAppArmorAccess
from bonesinfra.services.linux.apparmor import app as apparmor_app

from .helpers import SRC_DIR, make_site_request


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
        jinja2.Environment(autoescape=True, loader=jinja2.FileSystemLoader(str(SRC_DIR / "bonesinfra/assets")))
        .get_template("apparmor/app-profile.j2")
        .render(seen["data"])
    )
    assert "/srv/sites/lawsnipe/releases/*/** r," in rendered
    assert "/var/log/bonesdeploy/lawsnipe/ rw," in rendered


@pytest.mark.parametrize(
    "case",
    [
        ("rails", RAILS_TEMPLATES, "rails", "ruby", "3.3.8", "ruby"),
        ("gunicorn", DJANGO_TEMPLATES, "django", "python", "3.14.0", "python"),
        ("next", NEXT_TEMPLATES, "next", "node", "24.19.0", "node"),
        ("nuxt", NUXT_TEMPLATES, "nuxt", "node", "24.19.0", "node"),
        ("sveltekit", SVELTEKIT_TEMPLATES, "sveltekit", "node", "24.19.0", "node"),
    ],
)
def test_render_profile_allows_only_the_exact_mise_runtime(case, monkeypatch):
    runtime, templates, template_dir, tool, version, executable = case
    ctx = _ctx()
    seen = {}

    monkeypatch.setattr(
        apparmor_app,
        "render",
        lambda _name, _src, _dest, **data: seen.update(data=data) or types.SimpleNamespace(changes=[]),
    )
    monkeypatch.setattr(apparmor_app.server, "shell", lambda **_kw: types.SimpleNamespace(changes=[]))

    access = MiseRuntimeAppArmorAccess(
        executable=f"/var/lib/bonesdeploy/mise/installs/{tool}/{version}/bin/{executable}",
        root=f"/var/lib/bonesdeploy/mise/installs/{tool}/{version}",
    )
    paths = ctx.paths_dict
    apparmor_app.render_profile(
        ctx,
        paths=paths,
        runtime=runtime,
        template_src=templates / "app-profile.j2",
        apparmor_exec_paths=[f"{paths['project_root']}/.bonesdeploy/runtimes/{tool}/bin/{executable}"],
        apparmor_writable_paths=[],
        apparmor_runtime_access=access,
    )

    rendered = (
        jinja2.Environment(
            autoescape=True,
            loader=jinja2.FileSystemLoader(str(SRC_DIR / f"bonesinfra/frameworks/{template_dir}/templates")),
        )
        .get_template("app-profile.j2")
        .render(seen["data"])
    )
    assert f"/srv/sites/lawsnipe/.bonesdeploy/runtimes/{tool}/bin/{executable} mrix," in rendered
    assert f"/var/lib/bonesdeploy/mise/installs/{tool}/{version}/bin/{executable} mrix," in rendered
    assert f"/var/lib/bonesdeploy/mise/installs/{tool}/{version}/** rm," in rendered
    assert "/var/lib/bonesdeploy/mise/**" not in rendered


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
        jinja2.Environment(
            autoescape=True,
            loader=jinja2.FileSystemLoader(str(SRC_DIR / "bonesinfra/frameworks/sveltekit/templates")),
        )
        .get_template("app-profile.j2")
        .render(seen["data"])
    )
    assert "/srv/sites/lawsnipe/shared/.env r," in rendered


@pytest.mark.parametrize(
    ("runtime", "templates"),
    [("next", NEXT_TEMPLATES), ("nuxt", NUXT_TEMPLATES), ("sveltekit", SVELTEKIT_TEMPLATES)],
)
def test_node_framework_profiles_execute_only_the_managed_site_node_link(runtime, templates, monkeypatch):
    ctx = _ctx()
    seen = {}

    monkeypatch.setattr(
        apparmor_app,
        "render",
        lambda _name, _src, _dest, **data: seen.update(data=data) or types.SimpleNamespace(changes=[]),
    )
    monkeypatch.setattr(apparmor_app.server, "shell", lambda **_kw: types.SimpleNamespace(changes=[]))

    paths = ctx.paths_dict
    node_binary = f"{paths['node_runtime']}/bin/node"
    apparmor_app.render_profile(
        ctx,
        paths=paths,
        runtime=runtime,
        template_src=templates / "app-profile.j2",
        apparmor_exec_paths=[node_binary],
        apparmor_writable_paths=[],
    )

    rendered = (
        jinja2.Environment(
            autoescape=True,
            loader=jinja2.FileSystemLoader(str(SRC_DIR / f"bonesinfra/frameworks/{runtime}/templates")),
        )
        .get_template("app-profile.j2")
        .render(seen["data"])
    )
    assert f"{node_binary} mrix," in rendered


def test_django_profile_permits_the_site_managed_python(monkeypatch):
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
        runtime="gunicorn",
        template_src=DJANGO_TEMPLATES / "app-profile.j2",
        apparmor_exec_paths=[f"{paths['project_root']}/.bonesdeploy/runtimes/python/bin/python"],
        apparmor_writable_paths=[],
    )

    rendered = (
        jinja2.Environment(
            autoescape=True,
            loader=jinja2.FileSystemLoader(str(SRC_DIR / "bonesinfra/frameworks/django/templates")),
        )
        .get_template("app-profile.j2")
        .render(seen["data"])
    )
    assert "/usr/bin/python3* rix," not in rendered
    assert "/srv/sites/lawsnipe/.bonesdeploy/runtimes/python/bin/python mrix," in rendered
    assert "/srv/sites/lawsnipe/releases/*/ r," in rendered
    assert "/srv/sites/lawsnipe/releases/*/** r," in rendered
    assert "/srv/sites/lawsnipe/releases/*/**.so* mr," in rendered


def test_rails_profile_permits_the_site_ruby_and_packaged_bundler(monkeypatch):
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
        runtime="rails",
        template_src=RAILS_TEMPLATES / "app-profile.j2",
        apparmor_exec_paths=rails_apparmor_exec_paths(paths, ctx.paths.ruby_runtime + "/bin/ruby"),
        apparmor_writable_paths=[],
    )

    rendered = (
        jinja2.Environment(
            autoescape=True,
            loader=jinja2.FileSystemLoader(str(SRC_DIR / "bonesinfra/frameworks/rails/templates")),
        )
        .get_template("app-profile.j2")
        .render(seen["data"])
    )
    assert "/usr/bin/env mrix," in rendered
    assert "/srv/sites/lawsnipe/.bonesdeploy/runtimes/ruby/bin/ruby mrix," in rendered
    assert "/srv/sites/lawsnipe/releases/*/vendor/bundle/bundler/bin/bundle mrix," in rendered
    assert "/srv/sites/lawsnipe/releases/*/vendor/bundle/ruby/*/bin/puma mrix," in rendered
    assert "/usr/bin/ruby* mrix," not in rendered
