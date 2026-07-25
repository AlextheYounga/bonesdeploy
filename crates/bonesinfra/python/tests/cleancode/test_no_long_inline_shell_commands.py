"""Test: no server.shell calls with inline command arrays longer than 2 items.

Long commands should live in dedicated scripts under assets/scripts/
and be invoked via server.script() or server.script_template().
"""

import ast
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
SRC_DIRS = ["src"]
IGNORE_DIRS = {"venv", ".venv", ".env", "node_modules", "dist", "build", "coverage", "__pycache__"}


def _find_source_files() -> list[Path]:
    files = []
    for d in SRC_DIRS:
        src = PROJECT_ROOT / d
        if not src.is_dir():
            continue
        files.extend(
            path
            for path in src.rglob("*.py")
            if not any(part in IGNORE_DIRS for part in path.relative_to(PROJECT_ROOT).parts)
        )
    return files


def _command_list_size(node: ast.AST) -> int | None:
    if isinstance(node, ast.List):
        return len(node.elts)
    return None


@pytest.mark.parametrize("filepath", _find_source_files(), ids=lambda p: str(p.relative_to(PROJECT_ROOT)))
def test_no_long_inline_shell_commands(filepath: Path) -> None:
    tree = ast.parse(filepath.read_text(), filename=str(filepath))
    violations: list[str] = []

    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        if not isinstance(node.func, ast.Attribute) or node.func.attr != "shell":
            continue

        for kw in node.keywords:
            if kw.arg != "commands":
                continue
            size = _command_list_size(kw.value)
            if size is not None and size > 2:
                violations.append(f"  L{kw.value.lineno}: commands list has {size} items (max 2)")

    assert not violations, (
        f"server.shell with long commands array(s) in {filepath.relative_to(PROJECT_ROOT)}:\n"
        + "\n".join(violations)
        + "\n  Convert these to assets/scripts/ .sh files and use server.script() or server.script_template()."
    )
