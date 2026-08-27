from pathlib import Path

import pytest

from bonesinfra.config.context import DeployContext
from bonesinfra.frameworks.custom import manifest as core_manifest, runtime as core_runtime
from bonesinfra.project import load_manifest, load_runtime

from .helpers import make_site_request


def _project(tmp_path: Path, *, template: str = "custom") -> DeployContext:
    (tmp_path / "infra/templates").mkdir(parents=True)
    return DeployContext.from_request(make_site_request(template=template))


def _custom(tmp_path: Path, filename: str, content: str) -> None:
    custom = tmp_path / "infra/custom"
    custom.mkdir(parents=True, exist_ok=True)
    (custom / "__init__.py").write_text("")
    (custom / filename).write_text(content)


def test_runtime_loader_supports_relative_imports_in_custom_provisioning(tmp_path: Path, monkeypatch):
    config = _project(tmp_path)
    monkeypatch.chdir(tmp_path)
    _custom(tmp_path, "value.py", "VALUE = 7\n")
    _custom(tmp_path, "runtime.py", "from . import value\ndef deploy(_ctx):\n    return value.VALUE\n")

    assert load_runtime(config).deploy(None) == 7


def test_runtime_loader_uses_project_template_root(tmp_path: Path, monkeypatch):
    config = DeployContext.from_request(make_site_request(template="next"))
    (tmp_path / "infra/templates").mkdir(parents=True)
    monkeypatch.chdir(tmp_path)

    runtime = load_runtime(config)

    assert str(runtime.TEMPLATES) == str(tmp_path / "infra/templates/frameworks/next")


def test_runtime_loader_reports_custom_syntax_error_with_path(tmp_path: Path, monkeypatch):
    config = _project(tmp_path)
    monkeypatch.chdir(tmp_path)
    _custom(tmp_path, "runtime.py", "def deploy(:\n")

    with pytest.raises(ImportError, match=r"infra/custom/runtime\.py"):
        load_runtime(config)


def test_runtime_loader_requires_custom_deploy_callable(tmp_path: Path, monkeypatch):
    config = _project(tmp_path)
    monkeypatch.chdir(tmp_path)
    _custom(tmp_path, "runtime.py", "deploy = 3\n")

    with pytest.raises(TypeError, match="deploy"):
        load_runtime(config)


def test_manifest_loader_requires_all_custom_manifest_callables(tmp_path: Path, monkeypatch):
    config = _project(tmp_path)
    monkeypatch.chdir(tmp_path)
    _custom(tmp_path, "manifest.py", "def artifacts(_ctx):\n    return []\n")

    with pytest.raises(TypeError, match="services"):
        load_manifest(config)


def test_local_runtime_composes_core_before_custom(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    config = _project(tmp_path)
    monkeypatch.chdir(tmp_path)
    _custom(
        tmp_path,
        "runtime.py",
        "from pathlib import Path\ndef deploy(ctx):\n    Path(ctx).write_text(Path(ctx).read_text() + 'custom')\n",
    )
    monkeypatch.setattr(
        core_runtime,
        "deploy",
        lambda ctx: Path(ctx).write_text(Path(ctx).read_text() + "core"),
    )
    marker = tmp_path / "order"
    marker.write_text("")

    load_runtime(config).deploy(marker)

    assert marker.read_text() == "corecustom"


def test_local_manifest_composes_managed_and_custom_entries(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    config = _project(tmp_path)
    monkeypatch.chdir(tmp_path)
    _custom(
        tmp_path,
        "manifest.py",
        """def artifacts(_ctx):
    return ["custom artifact"]

def services(_ctx):
    return ["custom service"]

def mode(_ctx):
    return "custom"
""",
    )
    monkeypatch.setattr(core_manifest, "artifacts", lambda _ctx: ["managed artifact"])
    monkeypatch.setattr(core_manifest, "services", lambda _ctx: ["managed service"])
    monkeypatch.setattr(core_manifest, "mode", lambda _ctx: "managed")

    manifest = load_manifest(config)

    assert manifest.artifacts(None) == ["managed artifact", "custom artifact"]
    assert manifest.services(None) == ["managed service", "custom service"]
    assert manifest.mode(None) == "custom"
