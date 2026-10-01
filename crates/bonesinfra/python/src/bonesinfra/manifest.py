from __future__ import annotations

import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path
from shlex import quote
from typing import Any, Literal

from pyinfra.context import ctx_host
from pyinfra.facts.files import Directory, File, Link, Socket
from pyinfra.facts.server import Command, Which
from pyinfra.facts.systemd import SystemdEnabled, SystemdStatus

from bonesinfra.config.context import DeployContext
from bonesinfra.pyinfra.operations import letsencrypt_cert_paths

ArtifactKind = Literal["file", "directory", "link", "socket"]
ArtifactState = Literal["present", "missing", "wrong-kind"]


@dataclass(frozen=True)
class Artifact:
    """A project-owned remote path declared by BonesInfra."""

    name: str
    path_key: str
    kind: ArtifactKind
    owner: str
    path: str | None = None

    @classmethod
    def at_path(cls, name: str, path: str, kind: ArtifactKind, owner: str) -> Artifact:
        return cls(name, "", kind, owner, path)


@dataclass(frozen=True)
class ResolvedArtifact:
    name: str
    path: str
    kind: ArtifactKind
    owner: str
    state: ArtifactState
    actual_kind: str | None = None


@dataclass(frozen=True)
class ManagedService:
    name: str
    unit: str
    owner: str


@dataclass(frozen=True)
class ResolvedService:
    name: str
    unit: str
    owner: str
    running: bool
    enabled: bool


@dataclass(frozen=True)
class DeletionArtifact:
    """A validated site-owned artifact that may be removed."""

    name: str
    path: str
    kind: ArtifactKind
    owner: str


@dataclass(frozen=True)
class DeletionService:
    """A validated site-owned systemd service that must stop before teardown."""

    name: str
    unit: str
    owner: str


@dataclass(frozen=True)
class DeletionPlan:
    """The secret-free deletion inventory supported by the current manifest."""

    artifacts: tuple[DeletionArtifact, ...]
    services: tuple[DeletionService, ...]


COMMON_ARTIFACTS = (
    Artifact("project root", "project_root", "directory", "setup"),
    Artifact("releases directory", "releases", "directory", "setup"),
    Artifact("shared directory", "shared", "directory", "setup"),
    Artifact("current release link", "current", "link", "deploy"),
    Artifact("placeholder release", "placeholder_release", "directory", "setup"),
    Artifact("placeholder web root", "placeholder_web_root", "directory", "setup"),
    Artifact("placeholder index", "placeholder_index", "file", "setup"),
    Artifact("project configuration directory", "conf_root", "directory", "runtime"),
)


def collect_artifacts(ctx: DeployContext, project_manifest: Any) -> tuple[Artifact, ...]:
    """Return the artifacts expected for the context's deployment strategy."""
    artifacts = list(COMMON_ARTIFACTS)
    artifacts.extend(
        Artifact.at_path(artifact.name, artifact.path, artifact.kind, artifact.owner)
        for artifact in runtime_deletion_artifacts(ctx)
    )
    artifacts.extend(Artifact.at_path(*spec) for spec in project_manifest.artifacts(ctx))

    if ctx.app.dns.domain:
        artifacts.append(Artifact("ACME webroot", "acme_webroot", "directory", "ssl"))
        certificate, key = letsencrypt_cert_paths(ctx.app.dns.domain)
        artifacts.append(Artifact.at_path("ACME certificate", certificate, "link", "ssl"))
        artifacts.append(Artifact.at_path("ACME certificate key", key, "link", "ssl"))

    return _deduplicate(artifacts)


def runtime_deletion_artifacts(ctx: DeployContext) -> tuple[DeletionArtifact, ...]:
    """Return resources installed by the shared per-site nginx runtime."""
    if ctx.runtime.backend == "docker" and ctx.runtime.compose_port is None:
        return ()
    paths = ctx.paths
    return (
        DeletionArtifact("nginx site configuration", paths.site_nginx_config, "file", "runtime"),
        DeletionArtifact("nginx site", paths.nginx_site_available, "file", "runtime"),
        DeletionArtifact("enabled nginx site", paths.nginx_site_enabled, "link", "runtime"),
        DeletionArtifact("site systemd target", paths.systemd_site_target, "file", "runtime"),
        DeletionArtifact("site systemd requirements", paths.systemd_site_target_requires, "directory", "runtime"),
        DeletionArtifact("site nginx systemd service", paths.systemd_site_nginx_service, "file", "runtime"),
        DeletionArtifact("site nginx systemd requirement", paths.systemd_site_nginx_requirement, "link", "runtime"),
        DeletionArtifact("nginx AppArmor profile", paths.nginx_apparmor_profile, "file", "runtime"),
        DeletionArtifact("runtime socket directory", paths.runtime_socket_dir, "directory", "runtime"),
        DeletionArtifact("runtime nginx directory", paths.runtime_nginx_dir, "directory", "runtime"),
        DeletionArtifact("runtime nginx socket", paths.runtime_nginx_socket, "socket", "runtime"),
        DeletionArtifact("runtime nginx PID", paths.runtime_nginx_pid, "file", "runtime"),
    )


def collect_services(ctx: DeployContext, project_manifest: Any) -> tuple[ManagedService, ...]:
    """Return the project-specific systemd services managed by BonesInfra."""
    services = []
    if ctx.runtime.backend != "docker" or ctx.runtime.compose_port is not None:
        services.append(ManagedService("site nginx", f"{ctx.app.project_name}-nginx.service", "runtime"))
    services.extend(
        ManagedService(name, unit.format(project=ctx.app.project_name), owner)
        for name, unit, owner in project_manifest.services(ctx)
    )
    return _deduplicate_services(services)


def resolve_artifacts(ctx: DeployContext, project_manifest: Any) -> tuple[Artifact, ...]:
    """Validate and return declarations whose keys resolve through DeploymentPaths."""
    paths = ctx.paths
    for artifact in collect_artifacts(ctx, project_manifest):
        _artifact_path(paths, artifact)
    return collect_artifacts(ctx, project_manifest)


def resolve_deletion_plan(ctx: DeployContext, project_manifest: Any) -> DeletionPlan:
    """Resolve the current manifest into the supported, safe deletion inventory."""
    artifacts = (
        *(
            DeletionArtifact(artifact.name, _artifact_path(ctx.paths, artifact), artifact.kind, artifact.owner)
            for artifact in collect_artifacts(ctx, project_manifest)
        ),
        DeletionArtifact("quick tunnel service", ctx.paths.systemd_cloudflared_service, "file", "tunnel"),
        DeletionArtifact("quick tunnel nginx route", ctx.paths.nginx_cloudflared_site_available, "file", "tunnel"),
        DeletionArtifact(
            "enabled quick tunnel nginx route", ctx.paths.nginx_cloudflared_site_enabled, "link", "tunnel"
        ),
    )
    services = (
        *(
            DeletionService(service.name, service.unit, service.owner)
            for service in collect_services(ctx, project_manifest)
        ),
        DeletionService("quick tunnel", f"{ctx.app.project_name}-cloudflared.service", "tunnel"),
    )
    for artifact in artifacts:
        _validate_deletion_artifact(ctx, artifact)
    for service in services:
        _validate_deletion_service(ctx, service)
    return DeletionPlan(artifacts, services)


def render_deletion_plan(plan: DeletionPlan) -> str:
    """Render a deletion preflight in the stable JSON format consumed by BonesDeploy."""
    return json.dumps(asdict(plan), sort_keys=True)


def parse_deletion_plan(value: str, ctx: DeployContext) -> DeletionPlan:
    """Parse and validate the opaque, persisted plan returned by BonesRemote."""
    try:
        data = json.loads(value)
        artifacts = tuple(DeletionArtifact(**entry) for entry in data["artifacts"])
        services = tuple(DeletionService(**entry) for entry in data["services"])
    except (KeyError, TypeError, json.JSONDecodeError) as error:
        raise ValueError("invalid persisted deletion plan") from error
    plan = DeletionPlan(artifacts, services)
    for artifact in plan.artifacts:
        _validate_deletion_artifact(ctx, artifact)
    for service in plan.services:
        _validate_deletion_service(ctx, service)
    return plan


def inspect_artifacts(ctx: DeployContext, host: Any, project_manifest: Any) -> list[ResolvedArtifact]:
    """Inspect declared paths using read-only PyInfra file facts."""
    paths = ctx.paths
    resolved = resolve_artifacts(ctx, project_manifest)
    return [_inspect_one(host, artifact, _artifact_path(paths, artifact)) for artifact in resolved]


def inspect_services(ctx: DeployContext, host: Any, project_manifest: Any) -> list[ResolvedService]:
    """Inspect declared project-specific systemd services without changing them."""
    return [_inspect_service(host, service) for service in collect_services(ctx, project_manifest)]


def report(
    ctx: DeployContext,
    entries: list[ResolvedArtifact],
    services: list[ResolvedService],
    project_manifest: Any,
    compose_runtime: dict[str, Any] | None = None,
) -> dict[str, Any]:
    template = ctx.runtime.data.get("template")

    ssl_entries = [entry for entry in entries if entry.owner == "ssl"]
    data = {
        "strategy": {
            "backend": ctx.runtime.backend,
            "framework": template or "none",
            "mode": project_manifest.mode(ctx),
            "ssl": bool(ssl_entries) and all(entry.state == "present" for entry in ssl_entries),
        },
        "entries": [asdict(entry) for entry in entries],
        "managed_services": [asdict(service) for service in services],
    }
    if ctx.runtime.backend == "docker":
        data["compose"] = {
            "project_name": f"bonesdeploy-{ctx.app.project_name}",
            "files": [],
            "services": [],
            "engine_available": None,
            "plugin_available": None,
            "ingress_port": ctx.runtime.compose_port,
            "wait_timeout": ctx.runtime.compose_wait_timeout,
            "security_mode": "reduced-guarantee",
            "security_warning": (
                "The project Compose file is trusted privileged input; native runtime isolation is not guaranteed. "
                "BonesDeploy does not add a Docker socket mount."
            ),
            "persistent_data": (
                "Compose named volumes persist across deploy and rollback; their data is not rolled back."
            ),
            **(compose_runtime or {}),
        }
    return data


def render(data: dict[str, Any]) -> str:
    """Serialize the inspected manifest for the public Rust CLI."""
    return json.dumps(data, sort_keys=True)


def inspect_for_runner(ctx: DeployContext, project_manifest: Any) -> dict[str, Any]:
    host = ctx_host.get()
    return report(
        ctx,
        inspect_artifacts(ctx, host, project_manifest),
        inspect_services(ctx, host, project_manifest),
        project_manifest,
        _inspect_compose_runtime(ctx, host),
    )


def _inspect_compose_runtime(ctx: DeployContext, host: Any) -> dict[str, Any] | None:
    if ctx.runtime.backend != "docker":
        return None
    engine = host.get_fact(Which, command="docker")
    plugin = host.get_fact(
        Command,
        command="docker compose version >/dev/null 2>&1 && printf available || printf unavailable",
    )
    status = host.get_fact(Command, command=f"bonesremote status --site {quote(ctx.app.project_name)}")
    if not status:
        compose = {"error": "bonesremote status is unavailable"}
    else:
        try:
            compose = json.loads(status).get("compose") or {}
        except (TypeError, json.JSONDecodeError):
            compose = {"error": "bonesremote returned invalid Compose status"}
    return {
        "engine_available": bool(engine),
        "plugin_available": plugin == "available",
        **compose,
    }


def _inspect_one(host: Any, artifact: Artifact, path: str) -> ResolvedArtifact:
    facts = {"file": File, "directory": Directory, "link": Link, "socket": Socket}
    expected_fact = facts[artifact.kind]
    if host.get_fact(expected_fact, path) not in (None, False):
        return ResolvedArtifact(artifact.name, path, artifact.kind, artifact.owner, "present")

    for actual_kind, fact in facts.items():
        if actual_kind != artifact.kind and host.get_fact(fact, path) not in (None, False):
            return ResolvedArtifact(artifact.name, path, artifact.kind, artifact.owner, "wrong-kind", actual_kind)
    return ResolvedArtifact(artifact.name, path, artifact.kind, artifact.owner, "missing")


def _inspect_service(host: Any, service: ManagedService) -> ResolvedService:
    status = host.get_fact(SystemdStatus, services=service.unit) or {}
    enabled = host.get_fact(SystemdEnabled, services=service.unit) or {}
    return ResolvedService(
        service.name,
        service.unit,
        service.owner,
        bool(status.get(service.unit)),
        bool(enabled.get(service.unit)),
    )


def _artifact_path(paths: Any, artifact: Artifact) -> str:
    if artifact.path is not None:
        return artifact.path
    value = getattr(paths, artifact.path_key, None)
    if not isinstance(value, str):
        raise TypeError(f"manifest artifact {artifact.name!r} references unknown path key {artifact.path_key!r}")
    return value


def _validate_deletion_artifact(ctx: DeployContext, artifact: DeletionArtifact) -> None:
    if artifact.kind not in {"file", "directory", "link", "socket"}:
        raise ValueError(f"unsupported deletion artifact kind: {artifact.kind}")
    path = Path(artifact.path)
    if not path.is_absolute() or ".." in path.parts:
        raise ValueError(f"unsafe deletion artifact path: {artifact.path}")
    if not any(_is_within(path, root) for root in _deletion_roots(ctx)) and not _is_derived_system_path(ctx, path):
        raise ValueError(f"deletion artifact is outside site-owned paths: {artifact.path}")


def _validate_deletion_service(ctx: DeployContext, service: DeletionService) -> None:
    project = ctx.app.project_name
    if not re.fullmatch(rf"{re.escape(project)}-[a-z0-9][a-z0-9_-]*\.service", service.unit):
        raise ValueError(f"unsafe deletion service: {service.unit}")


def _deletion_roots(ctx: DeployContext) -> tuple[Path, ...]:
    paths = ctx.paths
    return tuple(
        Path(path)
        for path in (
            paths.project_root,
            paths.conf_root,
            paths.site_root,
            paths.runtime_socket_dir,
            paths.site_log_dir,
            paths.acme_webroot,
            paths.backup_repository,
        )
    )


def _is_within(path: Path, root: Path) -> bool:
    return path == root or root in path.parents


def _is_derived_system_path(ctx: DeployContext, path: Path) -> bool:
    project = ctx.app.project_name
    value = str(path)
    allowed = {
        ctx.paths.nginx_site_available,
        ctx.paths.nginx_site_enabled,
        ctx.paths.systemd_site_target,
        ctx.paths.systemd_site_target_requires,
        ctx.paths.systemd_site_nginx_service,
        ctx.paths.systemd_site_nginx_requirement,
        ctx.paths.systemd_cloudflared_service,
        ctx.paths.nginx_cloudflared_site_available,
        ctx.paths.nginx_cloudflared_site_enabled,
        ctx.paths.nginx_apparmor_profile,
        ctx.paths.backup_cron_file,
    }
    if value in allowed or _is_within(path, Path(ctx.paths.systemd_site_target_requires)):
        return True
    if value in letsencrypt_cert_paths(ctx.app.dns.domain):
        return True
    if any(
        value.startswith(prefix)
        for prefix in (
            f"/etc/systemd/system/{project}-",
            f"/etc/apparmor.d/bonesdeploy-{project}-",
            f"/etc/bonesinfra/services/{project}-",
        )
    ):
        return True
    escaped_project = re.escape(project)
    if re.fullmatch(rf"/etc/php/[0-9]+(?:\.[0-9]+)?/fpm/pool\.d/{escaped_project}\.conf", value):
        return True
    if re.fullmatch(rf"/run/php/php[0-9]+(?:\.[0-9]+)?-fpm-{escaped_project}\.sock", value):
        return True
    return any(_is_within(path, Path(f"/var/lib/{service}/{project}")) for service in _RUNTIME_DATA_SERVICES)


_RUNTIME_DATA_SERVICES = ("mysql", "mariadb", "mongodb", "redis", "valkey")


def _deduplicate(artifacts: list[Artifact]) -> tuple[Artifact, ...]:
    seen: set[str] = set()
    result = []
    for artifact in artifacts:
        identity = artifact.path or artifact.path_key
        if identity not in seen:
            result.append(artifact)
            seen.add(identity)
    return tuple(result)


def _deduplicate_services(services: list[ManagedService]) -> tuple[ManagedService, ...]:
    seen: set[str] = set()
    result = []
    for service in services:
        if service.unit not in seen:
            result.append(service)
            seen.add(service.unit)
    return tuple(result)
