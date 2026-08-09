from pathlib import Path

import pytest
from pyinfra.facts.files import Directory, Link
from pyinfra.facts.systemd import SystemdEnabled, SystemdStatus

from bonesinfra import manifest
from bonesinfra.config.context import DeployContext
from bonesinfra.frameworks import get_framework
from bonesinfra.manifest import (
    Artifact,
    collect_services,
    inspect_artifacts,
    inspect_services,
    render,
    report,
    resolve_artifacts,
)


def _context(
    tmp_path: Path,
    *,
    template: str = "",
    services: list[str] | None = None,
    ssl: bool = False,
    domain: str = "",
):
    config = tmp_path / "bones.toml"
    config.write_text(
        f"""[app]
project_name = "example"
[app.server]
host = "example.test"
[app.dns]
ssl_enabled = {str(ssl).lower()}
domain = "{domain}"
[runtime]
template = "{template}"
[services]
services = {services or []}
"""
    )
    return DeployContext.from_files(str(config))


def test_resolve_artifacts_selects_strategy_paths(tmp_path: Path):
    ctx = _context(tmp_path, template="next", services=["postgres"], ssl=True)

    artifacts = resolve_artifacts(ctx)
    by_name = {artifact.name: artifact for artifact in artifacts}

    assert by_name["placeholder release"].path_key == "placeholder_release"
    assert by_name["ACME webroot"].path_key == "acme_webroot"
    assert "postgres" in ctx.services.services


def test_server_framework_does_not_report_static_placeholder_index(tmp_path: Path):
    ctx = _context(tmp_path, template="next")
    ctx.runtime.data["is_static"] = False

    assert "static placeholder index" not in {artifact.name for artifact in resolve_artifacts(ctx)}


def test_resolve_artifacts_rejects_unknown_path_key(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    ctx = _context(tmp_path)
    monkeypatch.setattr(
        manifest,
        "COMMON_ARTIFACTS",
        (*manifest.COMMON_ARTIFACTS, Artifact("invalid", "not_a_path", "file", "test")),
    )

    with pytest.raises(TypeError, match="unknown path key"):
        resolve_artifacts(ctx)


def test_inspection_reports_present_missing_and_wrong_kind_without_contents(tmp_path: Path):
    ctx = _context(tmp_path)

    class FakeHost:
        def get_fact(self, fact, path):
            if path == ctx.paths.repo_head and fact is Directory:
                return {"mode": 755}
            if path == ctx.paths.repo and fact is Directory:
                return {"mode": 755}
            if path == ctx.paths.site_nginx_config:
                return None
            if fact is Directory and path == ctx.paths.project_root:
                return {"mode": 755}
            return None

    entries = inspect_artifacts(ctx, FakeHost())
    by_name = {entry.name: entry for entry in entries}

    assert by_name["bare repository"].state == "present"
    assert by_name["bare repository HEAD"].state == "wrong-kind"
    assert by_name["bare repository HEAD"].actual_kind == "directory"
    assert by_name["nginx site configuration"].state == "missing"

    data = report(ctx, [by_name["nginx site configuration"]], [])
    for output_format in ("text", "json"):
        output = render(data, output_format)
        assert "contents" not in output


def test_link_fact_is_reported_as_present(tmp_path: Path):
    ctx = _context(tmp_path)

    class FakeHost:
        def get_fact(self, fact, path):
            if path == ctx.paths.current and fact is Link:
                return {"link_target": "/srv/sites/example/releases/current"}
            return None

    entry = next(entry for entry in inspect_artifacts(ctx, FakeHost()) if entry.name == "current release link")
    assert entry.state == "present"


def test_framework_manifest_artifacts_and_mode(tmp_path: Path):
    ctx = _context(tmp_path, template="next")
    framework = get_framework("next")

    names = {spec[0] for spec in framework.manifest_artifacts(ctx)}
    assert names == {"current static web root", "static placeholder web root", "static placeholder index"}
    assert framework.manifest_mode(ctx) == "static"

    ctx.runtime.data["is_static"] = False
    server_names = {spec[0] for spec in framework.manifest_artifacts(ctx)}
    assert {
        "application AppArmor profile",
        "application systemd service",
        "application systemd requirement",
        "application runtime directory",
        "application runtime socket",
        "application log directory",
        "Next.js placeholder standalone directory",
        "Next.js placeholder standalone server",
    } == server_names
    assert framework.manifest_mode(ctx) == "server"

    assert get_framework("laravel").manifest_mode(ctx) == "php"


def test_manifest_includes_site_owned_systemd_apparmor_runtime_and_ssl_artifacts(tmp_path: Path):
    ctx = _context(tmp_path, template="next", services=["redis"], ssl=True, domain="example.test")
    ctx.runtime.data["is_static"] = False

    by_name = {artifact.name: artifact for artifact in resolve_artifacts(ctx)}

    assert by_name["site nginx AppArmor profile"].path_key == "nginx_apparmor_profile"
    assert ctx.paths.nginx_apparmor_profile == "/etc/apparmor.d/bonesdeploy-example-nginx"
    assert by_name["application systemd service"].path == "/etc/systemd/system/example-next.service"
    assert by_name["application systemd requirement"].path.endswith("example.target.requires/example-next.service")
    assert by_name["application runtime socket"].path == "/run/example/next/next.sock"
    assert by_name["redis configuration"].path == "/etc/bonesinfra/services/example-redis.conf"
    assert by_name["redis systemd service"].path == "/etc/systemd/system/example-redis.service"
    assert by_name["ACME certificate"].path == "/etc/letsencrypt/live/example.test/fullchain.pem"
    assert by_name["ACME certificate key"].path == "/etc/letsencrypt/live/example.test/privkey.pem"


def test_manifest_inspects_site_managed_services_without_mutations(tmp_path: Path):
    ctx = _context(tmp_path, template="next", services=["redis"])
    ctx.runtime.data["is_static"] = False

    class FakeHost:
        def get_fact(self, fact, *, services):
            values = {
                SystemdStatus: {
                    "example-nginx.service": True,
                    "example-next.service": False,
                    "example-redis.service": True,
                },
                SystemdEnabled: {
                    "example-nginx.service": False,
                    "example-next.service": False,
                    "example-redis.service": True,
                },
            }
            return {services: values[fact][services]}

    declared = {service.unit for service in collect_services(ctx)}
    assert declared == {
        "example-nginx.service",
        "example-next.service",
        "example-redis.service",
    }

    by_unit = {service.unit: service for service in inspect_services(ctx, FakeHost())}
    assert by_unit["example-next.service"].running is False
    assert by_unit["example-redis.service"].running is True
    assert by_unit["example-nginx.service"].enabled is False
    data = report(ctx, [], list(by_unit.values()))
    assert "Managed services:" in render(data, "text")
    assert '"managed_services"' in render(data, "json")
