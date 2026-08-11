from __future__ import annotations

from shlex import quote

from pyinfra.operations import files, server

from bonesinfra.config.context import DeployContext
from bonesinfra.config.paths import ASSETS_DIR, BONESREMOTE_CONFIG_DIR, BONESREMOTE_REPOS_DIR, DEFAULT_REPO_PARENT


def apply(ctx: DeployContext, patch_id: str) -> None:
    paths = ctx.paths
    repository = paths.bones_repo
    legacy_repository = f"{DEFAULT_REPO_PARENT}/{ctx.app.project_name}.bones.git"
    marker = f"/var/lib/bonesdeploy/patches/{ctx.app.project_name}/{patch_id}"
    repository_parent = f"{BONESREMOTE_CONFIG_DIR}/{BONESREMOTE_REPOS_DIR}"

    server.script_template(
        name=f"Migrate legacy .bones repository for {patch_id}",
        src=str(ASSETS_DIR / "scripts/migrate-config-repo-patch.sh.j2"),
        repository=quote(repository),
        legacy_repository=quote(legacy_repository),
        repository_parent=quote(repository_parent),
        _sudo=True,
    )
    files.put(
        name=f"Install .bones pre-receive hook for {patch_id}",
        src=str(ASSETS_DIR / "hooks/config-pre-receive"),
        dest=f"{repository}/hooks/pre-receive",
        user="root",
        group="root",
        mode="0755",
        _sudo=True,
    )
    server.shell(
        name=f"Write remote patch marker {patch_id}",
        commands=[
            (
                f"if [ ! -e {quote(marker)} ]; then mkdir -p {quote(marker.rsplit('/', 1)[0])}; "
                f"tmp={quote(marker)}.tmp-$$; printf 'completed\\n' > \"$tmp\"; "
                f'mv "$tmp" {quote(marker)}; fi'
            ),
        ],
        _sudo=True,
    )
