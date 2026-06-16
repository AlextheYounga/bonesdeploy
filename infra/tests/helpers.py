import os
import sys
from pathlib import Path

INFRA_DIR = Path(__file__).resolve().parent.parent
SRC_DIR = INFRA_DIR / "src"
REPO_ROOT = INFRA_DIR.parent
sys.path.insert(0, str(INFRA_DIR))


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
    import subprocess

    result = subprocess.run(
        [sys.executable, *args],
        capture_output=True,
        text=True,
        timeout=10,
        env={**os.environ, "PYTHONPATH": str(INFRA_DIR)},
    )
    assert result.returncode == 0, f"Failed: {' '.join(args)}\n{result.stderr}"
    return result.stdout
