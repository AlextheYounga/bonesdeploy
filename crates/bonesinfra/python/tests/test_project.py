from pathlib import Path

import pytest

from bonesinfra.project import load_manifest, load_runtime, materialize


def _project(tmp_path: Path, *, runtime: str = "def deploy(_ctx):\n    pass\n", manifest: str | None = None):
    core = tmp_path / "infra" / "provision" / "core"
    core.mkdir(parents=True)
    (core / "__init__.py").write_text("")
    (core / "custom.py").write_text("VALUE = 7\n")
    (core / "runtime.py").write_text(runtime)
    if manifest is not None:
        (core / "manifest.py").write_text(manifest)
    config = tmp_path / ".env"
    config.write_text("")
    return config


def test_runtime_loader_supports_relative_imports(tmp_path: Path):
    config = _project(tmp_path, runtime="from . import custom\ndef deploy(_ctx):\n    return custom.VALUE\n")
    module = load_runtime(config)
    assert module.deploy(None) == 7


def test_runtime_loader_reports_missing_file(tmp_path: Path):
    config = tmp_path / ".env"
    config.write_text("")
    (tmp_path / "infra/provision/core").mkdir(parents=True)
    with pytest.raises(FileNotFoundError, match=r"infra/provision/core/runtime\.py"):
        load_runtime(config)


def test_runtime_loader_reports_syntax_error_with_path(tmp_path: Path):
    config = _project(tmp_path, runtime="def deploy(:\n")
    with pytest.raises(ImportError, match=r"infra/provision/core/runtime\.py"):
        load_runtime(config)


def test_runtime_loader_requires_deploy_callable(tmp_path: Path):
    config = _project(tmp_path, runtime="deploy = 3\n")
    with pytest.raises(TypeError, match="deploy"):
        load_runtime(config)


def test_manifest_loader_requires_all_manifest_callables(tmp_path: Path):
    config = _project(tmp_path, manifest="def artifacts(_ctx):\n    return []\n")
    with pytest.raises(TypeError, match="services"):
        load_manifest(config)


def test_missing_local_package_uses_selected_builtin_framework(tmp_path: Path):
    config = tmp_path / ".env"
    config.write_text("TEMPLATE=next\n")

    module = load_runtime(config)

    assert module.__name__ == "bonesinfra.frameworks.next.runtime"


def test_materialize_copies_canonical_framework_package(tmp_path: Path):
    config = tmp_path / ".env"
    config.write_text("TEMPLATE=django\n")

    destination = materialize(config)

    assert destination == tmp_path / "infra/provision"
    assert (destination / "core/runtime.py").is_file()
    assert (destination / "core/manifest.py").is_file()
    assert (destination / "core/templates/django/placeholder-wsgi.py.j2").is_file()
    assert (destination / "custom/runtime.py").is_file()


def test_local_runtime_composes_core_before_custom(tmp_path: Path):
    config = _project(
        tmp_path,
        runtime="\n".join(
            (
                "from pathlib import Path",
                "def deploy(_ctx):",
                "    Path(_ctx).write_text(Path(_ctx).read_text() + 'core')",
            )
        ),
    )
    custom = tmp_path / "infra/provision/custom"
    custom.mkdir(parents=True)
    (custom / "__init__.py").write_text("")
    (custom / "runtime.py").write_text(
        "from pathlib import Path\ndef deploy(ctx):\n    Path(ctx).write_text(Path(ctx).read_text() + 'custom')\n"
    )
    marker = tmp_path / "order"
    marker.write_text("")

    load_runtime(config).deploy(marker)

    assert marker.read_text() == "corecustom"


def test_local_manifest_composes_managed_and_custom_entries(tmp_path: Path):
    config = _project(
        tmp_path,
        manifest="""def artifacts(_ctx):
    return [\"managed artifact\"]

def services(_ctx):
    return [\"managed service\"]

def mode(_ctx):
    return \"managed\"
""",
    )
    custom = tmp_path / "infra/provision/custom"
    custom.mkdir(parents=True)
    (custom / "__init__.py").write_text("")
    (custom / "manifest.py").write_text(
        """def artifacts(_ctx):
    return [\"custom artifact\"]

def services(_ctx):
    return [\"custom service\"]

def mode(_ctx):
    return \"custom\"
"""
    )

    manifest = load_manifest(config)

    assert manifest.artifacts(None) == ["managed artifact", "custom artifact"]
    assert manifest.services(None) == ["managed service", "custom service"]
    assert manifest.mode(None) == "custom"
