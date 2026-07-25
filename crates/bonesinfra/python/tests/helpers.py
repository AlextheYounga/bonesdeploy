import os
import subprocess
import sys
from functools import cache
from pathlib import Path

INFRA_DIR = Path(__file__).resolve().parent.parent
SRC_DIR = INFRA_DIR / "src"
REPO_ROOT = INFRA_DIR.parent
sys.path.insert(0, str(SRC_DIR))

PYTHON_BIN = sys.executable
PYTHON_ENV = {**os.environ, "PYTHONPATH": str(SRC_DIR)}


@cache
def read(path):
    return Path(path).read_text()


def assert_contains(text, pattern, msg=""):
    assert pattern in text, f"{msg}\n  missing: {pattern!r}"


def assert_not_contains(text, pattern, msg=""):
    assert pattern not in text, f"{msg}\n  unexpected: {pattern!r}"


def assert_ordering(text, *anchors):
    idx = -1
    for anchor in anchors:
        new_idx = text.find(anchor, idx + 1)
        assert new_idx > idx, f"Must appear in order, missing earlier: {anchor!r}"


def assert_file_exists(path, msg=""):
    assert Path(path).exists(), msg or f"Missing file: {path}"


def assert_file_not_exists(path, msg=""):
    assert not Path(path).exists(), msg or f"Unexpected file: {path}"


def compile_module(path):
    source = Path(path).read_text()
    return compile(source, str(path), "exec")


def exec_module(path):
    source = Path(path).read_text()
    ns = {}
    exec(source, ns)
    return ns


def run(*args):
    result = subprocess.run(
        [sys.executable, "-m", "bonesinfra", *args],
        capture_output=True,
        text=True,
        timeout=10,
        env=PYTHON_ENV,
        check=False,
    )
    assert result.returncode == 0, f"Failed: {' '.join(args)}\n{result.stderr}"
    return result.stdout
