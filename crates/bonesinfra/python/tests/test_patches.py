from pathlib import Path

import pytest

from bonesinfra.config.context import DeployContext
from bonesinfra.patches import remote
from bonesinfra.patches.registry import Version, apply_local, select_patches


def _context(tmp_path: Path) -> tuple[DeployContext, Path]:
    env_file = tmp_path / ".env"
    env_file.write_text("PROJECT_NAME=atlas\n")
    return DeployContext.from_files(str(env_file)), env_file


def test_project_infra_patch_selects_only_080_and_later():
    assert select_patches("0.7.7") == ()
    assert [patch.identifier for patch in select_patches("0.8.0-rc.1")] == ["0003-project-infra"]
    assert Version.parse("0.8.0+build") == Version(0, 8, 0)


def test_local_patch_migrates_owned_content_and_preserves_ciphertext(tmp_path, monkeypatch):
    ctx, env_file = _context(tmp_path)
    old = tmp_path / ".bones"
    (old / "infra/templates").mkdir(parents=True)
    (old / "deployment/build").mkdir(parents=True)
    (old / "secrets").mkdir(parents=True)
    (old / "infra/templates/site.conf").write_text("template")
    (old / "deployment/build/01_build.sh").write_text("build")
    ciphertext = bytes((0, 159, 42, 255))
    (old / "secrets/.env.gpg").write_bytes(ciphertext)
    (old / "bones.toml").write_text("must not be copied")
    monkeypatch.setenv("XDG_DATA_HOME", str(tmp_path / "data"))

    apply_local(ctx, "0.8.0", str(env_file))

    assert (tmp_path / "infra/secrets/.env.gpg").read_bytes() == ciphertext
    assert (tmp_path / "infra/custom/templates/site.conf").is_file()
    assert (tmp_path / "infra/custom/__init__.py").is_file()
    assert (tmp_path / "infra/deployment/build/01_build.sh").is_file()
    assert not old.exists()
    assert not (tmp_path / "infra/custom/bones.toml").exists()
    assert (tmp_path / "data/bonesdeploy/patches/atlas/0003-project-infra").is_file()


def test_local_patch_preserves_pre_materialized_framework(tmp_path, monkeypatch):
    ctx, env_file = _context(tmp_path)
    (tmp_path / ".bones/infra").mkdir(parents=True)
    (tmp_path / ".bones/infra/runtime.py").write_text("def deploy(_ctx):\n    pass\n")
    core = tmp_path / "infra/.framework"
    core.mkdir(parents=True)
    (core / "managed.py").write_text("managed")
    monkeypatch.setenv("XDG_DATA_HOME", str(tmp_path / "data"))

    apply_local(ctx, "0.8.0", str(env_file))

    assert (core / "managed.py").read_text() == "managed"
    assert (tmp_path / "infra/custom/runtime.py").is_file()
    assert not (tmp_path / ".bones").exists()
    assert (tmp_path / "data/bonesdeploy/patches/atlas/0003-project-infra").is_file()


def test_local_patch_refuses_custom_collision_without_writing_marker(tmp_path, monkeypatch):
    ctx, env_file = _context(tmp_path)
    (tmp_path / ".bones/infra").mkdir(parents=True)
    (tmp_path / ".bones/infra/runtime.py").write_text("def deploy(_ctx):\n    pass\n")
    (tmp_path / "infra/custom").mkdir(parents=True)
    monkeypatch.setenv("XDG_DATA_HOME", str(tmp_path / "data"))

    with pytest.raises(RuntimeError, match="will not merge or overwrite"):
        apply_local(ctx, "0.8.0", str(env_file))

    assert (tmp_path / ".bones").is_dir()
    assert not (tmp_path / "data/bonesdeploy/patches/atlas/0003-project-infra").exists()


def test_remote_patch_writes_a_per_project_completion_marker(tmp_path, monkeypatch):
    ctx, _ = _context(tmp_path)
    operations = []
    monkeypatch.setattr(remote.server, "shell", lambda **kwargs: operations.append(kwargs))

    remote.write_marker(ctx, "0003-project-infra")

    assert operations[0]["_sudo"] is True
    assert "/var/lib/bonesdeploy/patches/atlas/0003-project-infra" in operations[0]["commands"][0]
