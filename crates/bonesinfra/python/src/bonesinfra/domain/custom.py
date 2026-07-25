"""Local-only ``.bones/custom.py`` extension hook loader.

The custom.py file lives next to ``bones.toml`` and is treated as trusted
arbitrary local code. It is loaded with :mod:`importlib` before opening an SSH
connection so syntax/import failures fail fast with the file path visible.

Only stdlib, pyinfra, and installed bonesinfra modules are importable from the
hook; no dependency installation mechanism is provided. Missing files and
missing hooks are no-ops; a present but non-callable hook is an explicit error.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from types import ModuleType
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from bonesinfra.domain.context import DeployContext

HOOK_NAMES = ("after_setup", "after_runtime", "after_ssl")


def custom_path_for(config_path: str | Path) -> Path:
    return Path(config_path).resolve().parent / "custom.py"


def load_custom_module(config_path: str | Path) -> ModuleType | None:
    """Load ``custom.py`` next to ``bones.toml`` before SSH connect.

    Returns ``None`` when the file is absent. Raises ``SystemExit`` on syntax,
    import, or hook-shape failures so the path is visible before any remote
    work begins.
    """
    path = custom_path_for(config_path)
    if not path.is_file():
        return None

    spec = importlib.util.spec_from_file_location("__bones_custom__", path)
    if spec is None or spec.loader is None:
        print(f"Error: cannot load custom hook file: {path}", file=sys.stderr)
        sys.exit(3)

    module = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(module)
    except Exception as exc:  # surface any load failure with the file path visible
        print(f"Error loading {path}: {exc}", file=sys.stderr)
        sys.exit(3)

    _validate_hooks(module, path)
    return module


def _validate_hooks(module: ModuleType, path: Path) -> None:
    for name in HOOK_NAMES:
        hook = getattr(module, name, None)
        if hook is not None and not callable(hook):
            print(f"Error: {name} in {path} must be callable", file=sys.stderr)
            sys.exit(3)


def call_hook(module: ModuleType | None, name: str, ctx: DeployContext) -> None:
    """Run ``name`` on ``module`` inside the active pyinfra planning context.

    No-op when the module or the hook is absent. The hook receives ``ctx``
    unchanged.
    """
    if module is None:
        return
    hook = getattr(module, name, None)
    if hook is None:
        return
    hook(ctx)
