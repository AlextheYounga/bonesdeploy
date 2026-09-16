from pyinfra.operations import files, systemd

from bonesinfra.manifest import DeletionPlan


def deploy_site_delete(_ctx, plan: DeletionPlan) -> None:
    """Stop declared services, then remove the validated manifest artifacts."""
    for service in plan.services:
        systemd.service(
            name=f"Stop and disable {service.name}",
            service=service.unit,
            running=False,
            enabled=False,
            _sudo=True,
        )
    for artifact in sorted(plan.artifacts, key=lambda item: len(item.path), reverse=True):
        _remove_artifact(artifact.name, artifact.path, artifact.kind)


def _remove_artifact(name: str, path: str, kind: str) -> None:
    if kind == "directory":
        files.directory(name=f"Remove {name}", path=path, present=False, recursive=True, _sudo=True)
    elif kind == "link":
        files.link(name=f"Remove {name}", path=path, present=False, _sudo=True)
    else:
        # files.file removes regular files and Unix sockets without following links.
        files.file(name=f"Remove {name}", path=path, present=False, _sudo=True)
