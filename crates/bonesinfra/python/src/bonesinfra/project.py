from __future__ import annotations

import hashlib
import importlib
import importlib.util
import sys
import tomllib
from pathlib import Path
from shutil import copyfile
from types import ModuleType
from typing import Any

BUILTIN_FRAMEWORKS = frozenset({"custom", "django", "laravel", "next", "nuxt", "rails", "sveltekit", "vue"})


def load_runtime(config_path: str | Path) -> ModuleType:
    return _load_selected(config_path, "runtime.py", "deploy")


def load_manifest(config_path: str | Path) -> ModuleType:
    module = _load_selected(config_path, "manifest.py", "artifacts")
    path = _module_path(config_path, module, "manifest.py")
    _require_callable(module, "services", path)
    _require_callable(module, "mode", path)
    return module


def materialize(config_path: str | Path, framework: str | None = None) -> Path:
    destination = _entrypoint_path(config_path, "runtime.py").parent
    if destination.exists():
        raise FileExistsError(f"project infrastructure directory already exists: {destination}")

    selected = framework or _selected_framework(config_path)
    if selected not in BUILTIN_FRAMEWORKS:
        raise ValueError(f"unknown framework infrastructure: {selected}")
    source = Path(__file__).parent / "frameworks" / selected
    if not source.is_dir():
        raise ValueError(f"unknown framework infrastructure: {selected}")

    for source_file in source.rglob("*"):
        if source_file.is_file():
            target = destination / source_file.relative_to(source)
            target.parent.mkdir(parents=True, exist_ok=True)
            copyfile(source_file, target)
    return destination


def _load_selected(config_path: str | Path, filename: str, callable_name: str) -> ModuleType:
    infrastructure = _entrypoint_path(config_path, filename).parent
    if infrastructure.exists():
        return _load_local_entrypoint(config_path, filename, callable_name)

    framework = _selected_framework(config_path)
    try:
        module = importlib.import_module(f"bonesinfra.frameworks.{framework}.{filename[:-3]}")
    except Exception as error:
        raise ImportError(f"failed to import built-in infrastructure for {framework}: {error}") from error
    _require_callable(module, callable_name, Path(module.__file__ or filename))
    return module


def _load_local_entrypoint(config_path: str | Path, filename: str, callable_name: str) -> ModuleType:
    path = _entrypoint_path(config_path, filename)
    if not path.is_file():
        raise FileNotFoundError(f"project infrastructure file does not exist: {path}")

    package_name = f"_bones_project_infra_{hashlib.sha256(str(path.parent).encode()).hexdigest()[:16]}"
    package_path = path.parent / "__init__.py"
    if not package_path.is_file():
        raise FileNotFoundError(f"project infrastructure package file does not exist: {package_path}")

    _import_module(package_name, package_path, package_name, is_package=True)
    module = _import_module(f"{package_name}.{path.stem}", path, package_name)
    _require_callable(module, callable_name, path)
    return module


def _selected_framework(config_path: str | Path) -> str:
    with Path(config_path).open("rb") as config_file:
        data = tomllib.load(config_file)
    selected = str(data.get("runtime", {}).get("template") or "custom")
    if selected not in BUILTIN_FRAMEWORKS:
        raise ValueError(f"unknown framework infrastructure: {selected}")
    return selected


def _module_path(config_path: str | Path, module: ModuleType, filename: str) -> Path:
    return Path(module.__file__) if module.__file__ else _entrypoint_path(config_path, filename)


def _entrypoint_path(config_path: str | Path, filename: str) -> Path:
    return Path(config_path).resolve().parent / "infra" / filename


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
