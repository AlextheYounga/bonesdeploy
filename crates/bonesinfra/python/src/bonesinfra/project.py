from __future__ import annotations

import hashlib
import importlib
import importlib.util
import sys
from pathlib import Path
from shutil import copyfile
from types import ModuleType
from typing import Any

BUILTIN_FRAMEWORKS = frozenset({"custom", "django", "laravel", "next", "nuxt", "rails", "sveltekit", "vue"})


def load_runtime(config_path: str | Path) -> ModuleType:
    core, custom = _load_selected(config_path, "runtime.py", "deploy")
    return _compose_runtime(core, custom)


def load_manifest(config_path: str | Path) -> ModuleType:
    core, custom = _load_selected(config_path, "manifest.py", "artifacts")
    core_path = _module_path(config_path, core, "manifest.py")
    _require_callable(core, "services", core_path)
    _require_callable(core, "mode", core_path)
    if custom is not None:
        custom_path = _module_path(config_path, custom, "manifest.py")
        _require_callable(custom, "services", custom_path)
        _require_callable(custom, "mode", custom_path)
    return _compose_manifest(core, custom)


def materialize(config_path: str | Path, framework: str | None = None) -> Path:
    destination = _provision_path(config_path)
    if destination.exists():
        raise FileExistsError(f"project infrastructure directory already exists: {destination}")

    selected = framework or _selected_framework(config_path)
    if selected not in BUILTIN_FRAMEWORKS:
        raise ValueError(f"unknown framework infrastructure: {selected}")
    source = Path(__file__).parent / "frameworks" / selected
    if not source.is_dir():
        raise ValueError(f"unknown framework infrastructure: {selected}")

    core = destination / "core"
    custom = destination / "custom"
    for source_file in source.rglob("*"):
        if source_file.is_file():
            target = core / source_file.relative_to(source)
            target.parent.mkdir(parents=True, exist_ok=True)
            copyfile(source_file, target)
    custom.mkdir(parents=True)
    (custom / "__init__.py").write_text('"""Project-owned provisioning."""\n')
    (custom / "runtime.py").write_text("def deploy(_ctx):\n    pass\n")
    (custom / "manifest.py").write_text(
        "def artifacts(_ctx):\n    return []\n\n"
        "def services(_ctx):\n    return []\n\n"
        "def mode(_ctx):\n    return None\n"
    )
    return destination


def _load_selected(config_path: str | Path, filename: str, callable_name: str) -> tuple[ModuleType, ModuleType | None]:
    provision = _provision_path(config_path)
    if provision.exists():
        core = _load_local_entrypoint(provision / "core", filename, callable_name, required=True)
        custom = _load_local_entrypoint(provision / "custom", filename, callable_name, required=False)
        return core, custom

    framework = _selected_framework(config_path)
    try:
        module = importlib.import_module(f"bonesinfra.frameworks.{framework}.{filename[:-3]}")
    except Exception as error:
        raise ImportError(f"failed to import built-in infrastructure for {framework}: {error}") from error
    _require_callable(module, callable_name, Path(module.__file__ or filename))
    return module, None


def _load_local_entrypoint(
    package_path: Path, filename: str, callable_name: str, *, required: bool
) -> ModuleType | None:
    path = package_path / filename
    if not path.is_file():
        if not required:
            return None
        raise FileNotFoundError(f"project infrastructure file does not exist: {path}")

    package_name = f"_bones_project_infra_{hashlib.sha256(str(package_path).encode()).hexdigest()[:16]}"
    package_path = path.parent / "__init__.py"
    if not package_path.is_file():
        raise FileNotFoundError(f"project infrastructure package file does not exist: {package_path}")

    _import_module(package_name, package_path, package_name, is_package=True)
    module = _import_module(f"{package_name}.{path.stem}", path, package_name)
    _require_callable(module, callable_name, path)
    return module


def _compose_runtime(core: ModuleType, custom: ModuleType | None) -> ModuleType:
    if custom is None:
        return core

    composed = ModuleType("bonesinfra.project.runtime")

    def deploy(ctx):
        core.deploy(ctx)
        if custom is not None:
            custom.deploy(ctx)

    composed.deploy = deploy
    return composed


def _compose_manifest(core: ModuleType, custom: ModuleType | None) -> ModuleType:
    if custom is None:
        return core

    composed = ModuleType("bonesinfra.project.manifest")

    def artifacts(ctx):
        return _combined_entries(core, custom, "artifacts", ctx)

    def services(ctx):
        return _combined_entries(core, custom, "services", ctx)

    def mode(ctx):
        value = core.mode(ctx)
        if custom is not None:
            custom_value = custom.mode(ctx)
            if custom_value is not None:
                value = custom_value
        return value

    composed.artifacts = artifacts
    composed.services = services
    composed.mode = mode
    return composed


def _combined_entries(core: ModuleType, custom: ModuleType | None, name: str, ctx: Any) -> list[Any]:
    values = list(getattr(core, name)(ctx))
    if custom is not None:
        values.extend(getattr(custom, name)(ctx))
    return values


def _selected_framework(config_path: str | Path) -> str:
    selected = "custom"
    for line in Path(config_path).read_text().splitlines():
        key, separator, value = line.partition("=")
        if separator and key.strip() == "TEMPLATE":
            selected = value.strip().strip('"') or "custom"
            break
    if selected not in BUILTIN_FRAMEWORKS:
        raise ValueError(f"unknown framework infrastructure: {selected}")
    return selected


def _module_path(config_path: str | Path, module: ModuleType, filename: str) -> Path:
    return Path(module.__file__) if module.__file__ else _entrypoint_path(config_path, filename)


def _entrypoint_path(config_path: str | Path, filename: str) -> Path:
    return Path(config_path).resolve().parent / "infra" / filename


def _provision_path(config_path: str | Path) -> Path:
    return Path(config_path).resolve().parent / "infra" / "provision"


def _import_module(name: str, path: Path, package_name: str, *, is_package: bool = False) -> ModuleType:
    locations = [str(path.parent)] if is_package else None
    spec = importlib.util.spec_from_file_location(name, path, submodule_search_locations=locations)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot create import spec for project infrastructure file: {path}")
    module = importlib.util.module_from_spec(spec)
    module.__package__ = package_name
    sys.modules[name] = module
    try:
        spec.loader.exec_module(module)
    except Exception as error:
        raise ImportError(f"failed to import project infrastructure file {path}: {error}") from error
    return module


def _require_callable(module: ModuleType, name: str, path: Path) -> Any:
    value = getattr(module, name, None)
    if not callable(value):
        raise TypeError(f"project infrastructure file {path} must define callable {name}()")
    return value
