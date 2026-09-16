from __future__ import annotations

import json
from dataclasses import asdict, dataclass
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


COMMON_ARTIFACTS = (
    Artifact("bare repository", "repo", "directory", "setup"),
    Artifact("bare repository HEAD", "repo_head", "file", "setup"),
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
    artifacts.extend(Artifact.at_path(*spec) for spec in project_manifest.artifacts(ctx))

    if ctx.app.dns.ssl_enabled and ctx.app.dns.domain:
        artifacts.append(Artifact("ACME webroot", "acme_webroot", "directory", "ssl"))
        certificate, key = letsencrypt_cert_paths(ctx.app.dns.domain)
        artifacts.append(Artifact.at_path("ACME certificate", certificate, "link", "ssl"))
        artifacts.append(Artifact.at_path("ACME certificate key", key, "link", "ssl"))

    return _deduplicate(artifacts)


def collect_services(ctx: DeployContext, project_manifest: Any) -> tuple[ManagedService, ...]:
    """Return the project-specific systemd services managed by BonesInfra."""
    services = []
    if ctx.runtime.backend != "docker" or ctx.runtime.compose_port is not None:
        services.append(ManagedService("site nginx", f"{ctx.app.project_name}-nginx.service", "runtime"))
    services.extend(
        ManagedService(name, unit.format(project=ctx.app.project_name), owner)
        for name, unit, owner in project_manifest.services(ctx)
    )
    if not ctx.app.dns.domain and (ctx.runtime.backend != "docker" or ctx.runtime.compose_port is not None):
        services.append(ManagedService("quick tunnel", f"{ctx.app.project_name}-cloudflared.service", "runtime"))
    return _deduplicate_services(services)


def resolve_artifacts(ctx: DeployContext, project_manifest: Any) -> tuple[Artifact, ...]:
    """Validate and return declarations whose keys resolve through DeploymentPaths."""
    paths = ctx.paths
    for artifact in collect_artifacts(ctx, project_manifest):
        _artifact_path(paths, artifact)
    return collect_artifacts(ctx, project_manifest)


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

    data = {
        "strategy": {
            "backend": ctx.runtime.backend,
            "framework": template or "none",
            "mode": project_manifest.mode(ctx),
            "ssl": ctx.app.dns.ssl_enabled,
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


def render_text(data: dict[str, Any]) -> str:
    strategy = data["strategy"]
    lines = [
        f"Framework: {strategy['framework']} ({strategy['mode']})",
        f"Runtime backend: {strategy['backend']}",
        f"SSL: {'enabled' if strategy['ssl'] else 'disabled'}",
        "",
        "Manifest:",
    ]
    for entry in data["entries"]:
        suffix = f" (actual: {entry['actual_kind']})" if entry["actual_kind"] else ""
        lines.append(f"- [{entry['state']}] {entry['path']} [{entry['kind']}] {entry['owner']}{suffix}")
    lines.extend(["", "Managed services:"])
    for service in data["managed_services"]:
        state = "running" if service["running"] else "stopped"
        enabled = "enabled" if service["enabled"] else "disabled"
        lines.append(f"- [{state}, {enabled}] {service['unit']} {service['owner']}")
    if compose := data.get("compose"):
        lines.extend(
            [
                "",
                f"Compose project: {compose['project_name']}",
                f"Compose files: {', '.join(compose['files']) or 'not available'}",
                f"Compose security: {compose['security_mode']}",
                f"Warning: {compose['security_warning']}",
                f"Persistent data: {compose['persistent_data']}",
            ]
        )
        lines.extend(f"- [{service['condition']}] {service['service']}" for service in compose["services"])
        if compose.get("error"):
            lines.append(f"- [error] {compose['error']}")
    return "\n".join(lines)


def render(data: dict[str, Any], output_format: str) -> str:
    if output_format == "json":
        return json.dumps(data, sort_keys=True)
    if output_format == "text":
        return render_text(data)
    raise ValueError(f"unsupported manifest format: {output_format}")


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
