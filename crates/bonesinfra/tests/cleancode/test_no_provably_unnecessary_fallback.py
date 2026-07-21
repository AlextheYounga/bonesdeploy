"""Test: no provably unnecessary fallback patterns.

Detects ``or`` expressions where the left side is a literal that is
always truthy, making the right-hand side (fallback) dead code.
"""

import ast
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
SRC_DIRS = ["src", "app", "lib"]
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


def _has_items(node: ast.expr) -> bool:
    match node:
        case ast.List(elts=elts) | ast.Tuple(elts=elts) | ast.Set(elts=elts):
            return len(elts) > 0
        case ast.Dict(keys=keys):
            return len(keys) > 0
        case _:
            return False


def _is_definitely_truthy(node: ast.expr) -> bool:
    result = False
    match node:
        case ast.Constant(value=bool() as value):
            result = value
        case ast.Constant(value=str() as value):
            result = len(value) > 0
        case ast.Constant(value=int() as value):
            result = value != 0
        case ast.Constant(value=float() as value):
            result = value != 0.0
        case _:
            result = _has_items(node)
    return result


_SOURCE_FILES = _find_source_files()


@pytest.mark.parametrize("filepath", _SOURCE_FILES, ids=lambda p: str(p.relative_to(PROJECT_ROOT)))
def test_no_provably_unnecessary_fallback(filepath: Path) -> None:
    code = filepath.read_text(encoding="utf-8")
    try:
        tree = ast.parse(code)
    except SyntaxError:
        pytest.skip(f"Cannot parse {filepath}")

    violations: list[str] = []

    for node in ast.walk(tree):
        if not isinstance(node, ast.BoolOp) or node.op.__class__ is not ast.Or:
            continue
        if len(node.values) < 2:
            continue
        left = node.values[0]
        if _is_definitely_truthy(left):
            violations.append(f"  L{left.lineno}: left side of `or` is always truthy")

    assert not violations, f"Unnecessary fallback(s) in {filepath.relative_to(PROJECT_ROOT)}:\n" + "\n".join(violations)
