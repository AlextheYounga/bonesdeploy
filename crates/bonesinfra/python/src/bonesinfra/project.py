from __future__ import annotations

import hashlib
import importlib.util
import sys
from pathlib import Path
from types import ModuleType
from typing import Any


def load_runtime(config_path: str | Path) -> ModuleType:
    return _load_entrypoint(config_path, "runtime.py", "deploy")


def load_manifest(config_path: str | Path) -> ModuleType:
    module = _load_entrypoint(config_path, "manifest.py", "artifacts")
    _require_callable(module, "services", _entrypoint_path(config_path, "manifest.py"))
    _require_callable(module, "mode", _entrypoint_path(config_path, "manifest.py"))
    return module


def _load_entrypoint(config_path: str | Path, filename: str, callable_name: str) -> ModuleType:
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
