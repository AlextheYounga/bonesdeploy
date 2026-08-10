from pathlib import Path

import pytest

from bonesinfra.project import load_manifest, load_runtime


def _project(tmp_path: Path, *, runtime: str = "def deploy(_ctx):\n    pass\n", manifest: str | None = None):
    infra = tmp_path / "infra"
    infra.mkdir()
    (infra / "__init__.py").write_text("")
    (infra / "custom.py").write_text("VALUE = 7\n")
    (infra / "runtime.py").write_text(runtime)
    if manifest is not None:
        (infra / "manifest.py").write_text(manifest)
    config = tmp_path / "bones.toml"
    config.write_text("")
    return config


def test_runtime_loader_supports_relative_imports(tmp_path: Path):
    config = _project(tmp_path, runtime="from . import custom\ndef deploy(_ctx):\n    return custom.VALUE\n")
    module = load_runtime(config)
    assert module.deploy(None) == 7


def test_runtime_loader_reports_missing_file(tmp_path: Path):
    config = tmp_path / "bones.toml"
    config.write_text("")
    with pytest.raises(FileNotFoundError, match=r"infra/runtime\.py"):
        load_runtime(config)


def test_runtime_loader_reports_syntax_error_with_path(tmp_path: Path):
    config = _project(tmp_path, runtime="def deploy(:\n")
    with pytest.raises(ImportError, match=r"infra/runtime\.py"):
        load_runtime(config)


def test_runtime_loader_requires_deploy_callable(tmp_path: Path):
    config = _project(tmp_path, runtime="deploy = 3\n")
    with pytest.raises(TypeError, match="deploy"):
        load_runtime(config)


def test_manifest_loader_requires_all_manifest_callables(tmp_path: Path):
    config = _project(tmp_path, manifest="def artifacts(_ctx):\n    return []\n")
    with pytest.raises(TypeError, match="services"):
        load_manifest(config)
