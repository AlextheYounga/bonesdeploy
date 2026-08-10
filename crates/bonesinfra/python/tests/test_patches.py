from pathlib import Path

import pytest

from bonesinfra.config.context import DeployContext
from bonesinfra.patches import local, remote
from bonesinfra.patches.registry import Version, apply_local, select_patches


def _context(tmp_path: Path) -> tuple[DeployContext, Path]:
    bones_dir = tmp_path / ".bones"
    bones_dir.mkdir()
    config_path = bones_dir / "bones.toml"
    config_path.write_text(
        """[app]
project_name = "atlas"

[app.server]
host = "example.test"
port = 2222
"""
    )
    return DeployContext.from_files(str(config_path)), config_path


def test_patch_selection_preserves_order_and_prerelease_normalization():
    assert select_patches("0.7.2") == ()
    assert [patch.identifier for patch in select_patches("0.7.3-rc1")] == [
        "0001-config-repo",
        "0002-root-config-repo",
    ]
    assert Version.parse("0.7.3+build") == Version(0, 7, 3)


def test_local_patches_migrate_config_repository_and_write_markers(tmp_path, monkeypatch):
    ctx, config_path = _context(tmp_path)
    monkeypatch.setenv("XDG_DATA_HOME", str(tmp_path / "data"))

    apply_local(ctx, "0.7.3", str(config_path))

    assert local._git_output(config_path.parent, "remote", "get-url", "origin").stdout.strip() == (
        "ssh://root@example.test:2222/root/.config/bonesremote/repos/atlas.bones.git"
    )
    marker_dir = tmp_path / "data/bonesdeploy/patches/atlas"
    assert (marker_dir / "0001-config-repo").is_file()
    assert (marker_dir / "0002-root-config-repo").is_file()


def test_first_local_patch_rejects_unexpected_origin(tmp_path, monkeypatch):
    ctx, config_path = _context(tmp_path)
    monkeypatch.setenv("XDG_DATA_HOME", str(tmp_path / "data"))
    local._run_git(config_path.parent, "init", "--initial-branch", "master")
    local._run_git(config_path.parent, "remote", "add", "origin", "ssh://wrong.example/repo.git")

    with pytest.raises(RuntimeError, match="origin points to"):
        apply_local(ctx, "0.7.3", str(config_path))

    assert not (tmp_path / "data/bonesdeploy/patches/atlas/0001-config-repo").exists()


def test_remote_plan_repeats_migration_for_legacy_markers(monkeypatch, tmp_path):
    ctx, _ = _context(tmp_path)
    operations = []
    monkeypatch.setattr(remote.server, "shell", lambda **kwargs: operations.append(("shell", kwargs)))
    monkeypatch.setattr(remote.server, "script_template", lambda **kwargs: operations.append(("script", kwargs)))
    monkeypatch.setattr(remote.files, "put", lambda **kwargs: operations.append(("put", kwargs)))

    remote.apply(ctx, "0001-config-repo")
    remote.apply(ctx, "0002-root-config-repo")

    assert len([kind for kind, _ in operations if kind == "put"]) == 2
    scripts = [kwargs for kind, kwargs in operations if kind == "script"]
    assert len(scripts) == 2
    assert scripts[0]["legacy_repository"] == "/home/git/atlas.bones.git"
    markers = [kwargs for kind, kwargs in operations if kind == "shell"]
    assert any("0002-root-config-repo" in kwargs["commands"][0] for kwargs in markers)
    hook = next(kwargs for kind, kwargs in operations if kind == "put")
    assert hook["dest"].endswith("hooks/pre-receive")
