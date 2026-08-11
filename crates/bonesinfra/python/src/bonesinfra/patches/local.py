from __future__ import annotations

import subprocess
from pathlib import Path

from bonesinfra.config.context import DeployContext


def apply_config_repo(bones_dir: Path, ctx: DeployContext) -> None:
    if not _is_repository(bones_dir):
        _run_git(bones_dir, "init", "--initial-branch", "master")

    remote_url = config_repo_url(ctx)
    origin = _git_output(bones_dir, "remote", "get-url", "origin")
    if origin.returncode == 0:
        actual_url = origin.stdout.strip()
        if actual_url != remote_url:
            raise RuntimeError(f"origin points to {actual_url}, expected {remote_url}")
        return
    _run_git(bones_dir, "remote", "add", "origin", remote_url)


def apply_root_config_repo(bones_dir: Path, ctx: DeployContext) -> None:
    remote_url = config_repo_url(ctx)
    if _git_output(bones_dir, "remote", "get-url", "origin").returncode == 0:
        _run_git(bones_dir, "remote", "set-url", "origin", remote_url)
    else:
        _run_git(bones_dir, "remote", "add", "origin", remote_url)


def config_repo_url(ctx: DeployContext) -> str:
    repository = ctx.paths.bones_repo
    if ctx.app.server.port == "22":
        return f"root@{ctx.app.server.host}:{repository}"
    return f"ssh://root@{ctx.app.server.host}:{ctx.app.server.port}{repository}"


def _is_repository(path: Path) -> bool:
    return _git_output(path, "rev-parse", "--git-dir").returncode == 0


def _git_output(path: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(  # noqa: S603
        ["git", "-C", str(path), *args],  # noqa: S607
        capture_output=True,
        text=True,
        check=False,
    )


def _run_git(path: Path, *args: str) -> None:
    result = _git_output(path, *args)
    if result.returncode:
        detail = result.stderr.strip()
        raise RuntimeError(f"git command failed: {detail}")
