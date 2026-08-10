from pathlib import Path

import pytest
from pyinfra.facts.files import Directory, Link
from pyinfra.facts.systemd import SystemdEnabled, SystemdStatus

from bonesinfra import manifest
from bonesinfra.config.context import DeployContext
from bonesinfra.manifest import (
    Artifact,
    collect_services,
    inspect_artifacts,
    inspect_services,
    render,
    report,
    resolve_artifacts,
)


class ProjectManifest:
    def artifacts(self, ctx):
        return [("project artifact", ctx.paths.site_nginx_config, "file", "framework")]

    def services(self, _ctx):
        return [("project service", "{project}-app.service", "framework")]

    def mode(self, _ctx):
        return "server"


def _context(tmp_path: Path, *, ssl: bool = False):
    config = tmp_path / "bones.toml"
    config.write_text(
        f"""[app]
project_name = "example"
[app.server]
host = "example.test"
[app.dns]
ssl_enabled = {str(ssl).lower()}
domain = "example.test"
[runtime]
template = "custom"
[services]
services = []
"""
    )
    return DeployContext.from_files(str(config))


def test_project_manifest_declarations_are_included(tmp_path: Path):
    ctx = _context(tmp_path, ssl=True)
    project = ProjectManifest()
    artifacts = resolve_artifacts(ctx, project)
    names = {artifact.name for artifact in artifacts}
    assert "project artifact" in names
    assert "ACME certificate" in names
    assert collect_services(ctx, project)[-1].unit == "example-app.service"
    assert report(ctx, [], [], project)["strategy"]["mode"] == "server"


def test_resolve_artifacts_rejects_unknown_path_key(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    ctx = _context(tmp_path)
    monkeypatch.setattr(
        manifest,
        "COMMON_ARTIFACTS",
        (*manifest.COMMON_ARTIFACTS, Artifact("invalid", "not_a_path", "file", "test")),
    )
    with pytest.raises(TypeError, match="unknown path key"):
        resolve_artifacts(ctx, ProjectManifest())


def test_inspection_reports_present_missing_and_wrong_kind_without_contents(tmp_path: Path):
    ctx = _context(tmp_path)
    project = ProjectManifest()

    class FakeHost:
        def get_fact(self, fact, path):
            if path == ctx.paths.repo_head and fact is Directory:
                return {"mode": 755}
            if path == ctx.paths.repo and fact is Directory:
                return {"mode": 755}
            if fact is Directory and path == ctx.paths.project_root:
                return {"mode": 755}
            return None

    entries = inspect_artifacts(ctx, FakeHost(), project)
    by_name = {entry.name: entry for entry in entries}
    assert by_name["bare repository"].state == "present"
    assert by_name["bare repository HEAD"].state == "wrong-kind"
    assert by_name["bare repository HEAD"].actual_kind == "directory"
    assert by_name["project artifact"].state == "missing"
    output = render(report(ctx, entries, [], project), "json")
    assert "contents" not in output


def test_link_fact_is_reported_as_present(tmp_path: Path):
    ctx = _context(tmp_path)

    class FakeHost:
        def get_fact(self, fact, path):
            if path == ctx.paths.current and fact is Link:
                return {"link_target": "/srv/sites/example/releases/current"}
            return None

    project = ProjectManifest()
    entry = next(entry for entry in inspect_artifacts(ctx, FakeHost(), project) if entry.name == "current release link")
    assert entry.state == "present"


def test_services_are_inspected_without_mutations(tmp_path: Path):
    ctx = _context(tmp_path)

    class FakeHost:
        def get_fact(self, fact, *, services):
            values = {
                SystemdStatus: {"example-nginx.service": True, "example-app.service": False},
                SystemdEnabled: {"example-nginx.service": False, "example-app.service": True},
            }
            return {services: values[fact][services]}

    by_unit = {service.unit: service for service in inspect_services(ctx, FakeHost(), ProjectManifest())}
    assert by_unit["example-app.service"].running is False
    assert by_unit["example-app.service"].enabled is True
