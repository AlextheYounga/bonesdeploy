from shlex import quote

from pyinfra.operations import files, server, systemd

from bonesinfra.manifest import ArtifactKind, DeletionPlan


def deploy_site_delete(_ctx, plan: DeletionPlan) -> None:
    """Stop declared services, then remove the validated manifest artifacts."""
    for service in plan.services:
        if service.owner == "tunnel":
            unit = quote(service.unit)
            server.shell(
                name=f"Stop and disable {service.name} when installed",
                commands=[f"if systemctl cat -- {unit} >/dev/null 2>&1; then systemctl disable --now -- {unit}; fi"],
                _sudo=True,
            )
            continue
        systemd.service(
            name=f"Stop and disable {service.name}",
            service=service.unit,
            running=False,
            enabled=False,
            _sudo=True,
        )
    for artifact in sorted(plan.artifacts, key=lambda item: len(item.path), reverse=True):
        _remove_artifact(artifact.name, artifact.path, artifact.kind)
    if any(artifact.owner == "tunnel" for artifact in plan.artifacts):
        systemd.daemon_reload(name="Reload systemd after Quick Tunnel removal", _sudo=True)
        systemd.service(name="Reload nginx after Quick Tunnel removal", service="nginx", reloaded=True, _sudo=True)


def _remove_artifact(name: str, path: str, kind: ArtifactKind) -> None:
    if kind == "directory":
        files.directory(name=f"Remove {name}", path=path, present=False, recursive=True, _sudo=True)
    else:
        server.shell(name=f"Remove {name}", commands=[f"rm -f -- {quote(path)}"], _sudo=True)
