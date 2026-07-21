"""Test: no suspicious fallback patterns.

Detects ``try/except`` blocks where the except handler returns a
success value without re-raising — silently manufacturing success
from an error path.
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


def _has_return(stmts: list[ast.stmt]) -> bool:
    stack: list[ast.AST] = list(stmts)
    while stack:
        node = stack.pop()
        if isinstance(node, ast.Return):
            return True
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                continue
            stack.append(child)
    return False


def _has_raise(stmts: list[ast.stmt]) -> bool:
    stack: list[ast.AST] = list(stmts)
    while stack:
        node = stack.pop()
        if isinstance(node, ast.Raise):
            return True
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                continue
            stack.append(child)
    return False


def _find_returns(stmts: list[ast.stmt]) -> list[ast.Return]:
    results: list[ast.Return] = []
    stack: list[ast.AST] = list(stmts)
    while stack:
        node = stack.pop()
        if isinstance(node, ast.Return):
            results.append(node)
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                continue
            stack.append(child)
    return results


_SOURCE_FILES = _find_source_files()


@pytest.mark.parametrize("filepath", _SOURCE_FILES, ids=lambda p: str(p.relative_to(PROJECT_ROOT)))
def test_no_suspicious_fallback(filepath: Path) -> None:
    code = filepath.read_text(encoding="utf-8")
    try:
        tree = ast.parse(code)
    except SyntaxError:
        pytest.skip(f"Cannot parse {filepath}")

    violations: list[str] = []

    for node in ast.walk(tree):
        if not isinstance(node, ast.Try):
            continue
        if not _has_return(list(node.body)):
            continue
        for handler in node.handlers:
            if _has_raise(list(handler.body)):
                continue
            violations.extend(
                f"  L{return_node.lineno}: except handler returns success without re-raise"
                for return_node in _find_returns(list(handler.body))
            )

    assert not violations, f"Suspicious fallback(s) in {filepath.relative_to(PROJECT_ROOT)}:\n" + "\n".join(violations)
