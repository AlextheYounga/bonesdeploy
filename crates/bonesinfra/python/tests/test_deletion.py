import pytest

from bonesinfra.cli.commands.site import delete
from bonesinfra.config.context import DeployContext
from bonesinfra.manifest import (
    DeletionArtifact,
    DeletionPlan,
    DeletionService,
    parse_deletion_plan,
    render_deletion_plan,
    resolve_deletion_plan,
)

from .helpers import make_site_request


class ProjectManifest:
    def artifacts(self, ctx):
        return [("site nginx config", ctx.paths.site_nginx_config, "file", "runtime")]

    def services(self, _ctx):
        return [("application", "{project}-app.service", "framework")]

    def mode(self, _ctx):
        return "server"


def _context() -> DeployContext:
    return DeployContext.from_request(make_site_request(project_name="example"))


def test_deletion_plan_serializes_only_validated_manifest_resources():
    plan = resolve_deletion_plan(_context(), ProjectManifest())

    data = render_deletion_plan(plan)

    assert '"path": "/srv/sites/example"' in data
    assert '"unit": "example-app.service"' in data
    assert '"unit": "example-cloudflared.service"' in data
    assert "bonesdeploy-cloudflared-example.conf" in data
    assert "password" not in data


def test_deletion_plan_rejects_custom_artifact_outside_site_owned_paths():
    class UnsafeManifest(ProjectManifest):
        def artifacts(self, _ctx):
            return [("shared nginx configuration", "/etc/nginx/nginx.conf", "file", "custom")]

    with pytest.raises(ValueError, match="outside site-owned paths"):
        resolve_deletion_plan(_context(), UnsafeManifest())


def test_deletion_plan_rejects_unknown_artifact_kind():
    class UnsafeManifest(ProjectManifest):
        def artifacts(self, ctx):
            return [("unknown", ctx.paths.project_root, "device", "custom")]

    with pytest.raises(ValueError, match="unsupported deletion artifact kind"):
        resolve_deletion_plan(_context(), UnsafeManifest())


def test_persisted_deletion_plan_is_revalidated_before_execution():
    plan = resolve_deletion_plan(_context(), ProjectManifest())

    parsed = parse_deletion_plan(render_deletion_plan(plan), _context())

    assert parsed == plan


def test_deletion_stops_services_before_removing_artifacts(monkeypatch: pytest.MonkeyPatch):
    calls = []
    monkeypatch.setattr(delete.systemd, "service", lambda **kwargs: calls.append(("service", kwargs["service"])))
    monkeypatch.setattr(delete.files, "directory", lambda **kwargs: calls.append(("directory", kwargs["path"])))
    monkeypatch.setattr(delete.files, "file", lambda **kwargs: calls.append(("file", kwargs["path"])))
    monkeypatch.setattr(delete.files, "link", lambda **kwargs: calls.append(("link", kwargs["path"])))
    plan = DeletionPlan(
        artifacts=(
            DeletionArtifact("site root", "/srv/sites/example", "directory", "setup"),
            DeletionArtifact("site config", "/srv/sites/example/config", "file", "framework"),
        ),
        services=(DeletionService("application", "example-app.service", "framework"),),
    )

    delete.deploy_site_delete(_context(), plan)

    assert calls == [
        ("service", "example-app.service"),
        ("file", "/srv/sites/example/config"),
        ("directory", "/srv/sites/example"),
    ]


def test_deletion_removes_unix_sockets_without_treating_them_as_regular_files(monkeypatch: pytest.MonkeyPatch):
    calls = []
    monkeypatch.setattr(delete.server, "shell", lambda **kwargs: calls.append(kwargs))
    plan = DeletionPlan(
        artifacts=(DeletionArtifact("application socket", "/run/example/app.sock", "socket", "framework"),),
        services=(),
    )

    delete.deploy_site_delete(_context(), plan)

    assert calls[0]["commands"] == ["rm -f -- /run/example/app.sock"]


def test_deletion_reloads_systemd_and_nginx_after_optional_tunnel_cleanup(monkeypatch: pytest.MonkeyPatch):
    calls = []
    monkeypatch.setattr(delete.systemd, "service", lambda **kwargs: calls.append(("service", kwargs)))
    monkeypatch.setattr(delete.systemd, "daemon_reload", lambda **kwargs: calls.append(("daemon-reload", kwargs)))
    monkeypatch.setattr(delete.server, "shell", lambda **kwargs: calls.append(("shell", kwargs)))
    monkeypatch.setattr(delete.files, "file", lambda **kwargs: calls.append(("file", kwargs)))
    monkeypatch.setattr(delete.files, "link", lambda **kwargs: calls.append(("link", kwargs)))
    plan = DeletionPlan(
        artifacts=(
            DeletionArtifact(
                "quick tunnel service", "/etc/systemd/system/example-cloudflared.service", "file", "tunnel"
            ),
            DeletionArtifact(
                "quick tunnel nginx route",
                "/etc/nginx/sites-enabled/bonesdeploy-cloudflared-example.conf",
                "link",
                "tunnel",
            ),
        ),
        services=(DeletionService("quick tunnel", "example-cloudflared.service", "tunnel"),),
    )

    delete.deploy_site_delete(_context(), plan)

    assert any(operation == "daemon-reload" for operation, _kwargs in calls)
    tunnel_stop = next(kwargs for operation, kwargs in calls if operation == "shell")
    assert "systemctl cat" in tunnel_stop["commands"][0]
    assert "systemctl disable --now" in tunnel_stop["commands"][0]
    assert any(
        operation == "service" and kwargs.get("service") == "nginx" and kwargs.get("reloaded") is True
        for operation, kwargs in calls
    )
