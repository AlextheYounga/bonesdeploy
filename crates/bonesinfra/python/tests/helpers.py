import json
import os
import subprocess
import sys
from collections.abc import Mapping
from copy import deepcopy
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


def make_server_request(**overrides):
    request = {"server": {"host": "example.com", "ssh_user": "root", "port": "2222"}}
    request["server"].update(overrides)
    return request


def make_site_request(**overrides):
    request = {
        "server": make_server_request()["server"],
        "site": {
            "project_name": "lawsnipe",
            "domain": "example.com",
            "preview_domain": "preview.example.com",
            "email": "ops@example.com",
            "ssl_enabled": True,
            "template": "custom",
            "backend": "native",
            "web_root": "dist",
            "branch": "main",
            "node_version": "22",
            "services": [],
            "extras": {},
        },
        "services": {},
    }
    credentials = overrides.pop("service_credentials", None)
    if credentials is not None:
        request["services"] = deepcopy(credentials)
    site_services = overrides.pop("site_services", None)
    if site_services is not None:
        request["site"]["services"] = deepcopy(site_services)
    for section in ("server", "site"):
        values = overrides.pop(section, None)
        if values is not None:
            request[section].update(deepcopy(values))
    values = overrides.pop("services", None)
    if isinstance(values, Mapping):
        request["services"].update(deepcopy(values))
    request["site"].update(deepcopy(overrides))
    return request


def run(*args, input_text=None):
    result = subprocess.run(
        [sys.executable, "-m", "bonesinfra", *args],
        capture_output=True,
        text=True,
        input=json.dumps(input_text) if isinstance(input_text, dict) else input_text,
        timeout=10,
        env=PYTHON_ENV,
        check=False,
    )
    assert result.returncode == 0, f"Failed: {' '.join(args)}\n{result.stderr}"
    return result.stdout
