"""CLI commands must run without crashing."""

import json
import subprocess

from . import helpers

MAIN = helpers.INFRA_DIR / "main.py"
PYTHON = helpers.PYTHON_BIN


def _run_no_input(*args):
    return subprocess.run(
        [PYTHON, str(MAIN), *args],
        capture_output=True,
        text=True,
        timeout=10,
        env=helpers.PYTHON_ENV,
    )


def _run_with_stdin(stdin_data, *args):
    return subprocess.run(
        [PYTHON, str(MAIN), *args],
        input=json.dumps(stdin_data) if isinstance(stdin_data, dict) else stdin_data,
        capture_output=True,
        text=True,
        timeout=10,
        env=helpers.PYTHON_ENV,
    )


def test_runtime_list():
    helpers.run(MAIN, "runtime", "list", "--json")


def test_runtime_questions_all():
    for name in ["django", "laravel", "next", "rails", "sveltekit", "vue"]:
        helpers.run(MAIN, "runtime", "questions", name, "--json")


def test_runtime_defaults():
    for name in ["django", "laravel", "next"]:
        helpers.run(MAIN, "runtime", "defaults", name, "--json")


def test_setup_apply_rejects_missing_host():
    result = _run_with_stdin({"ssh_user": "root"}, "setup", "apply", "--config", "/dev/null")
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()


def test_runtime_apply_requires_ssh_user():
    result = _run_no_input("runtime", "apply", "--config", "/dev/null", "--runtime-config", "/dev/null")
    assert result.returncode != 0, f"Expected non-zero exit for missing --ssh-user"


def test_ssl_apply_rejects_missing_host():
    result = _run_with_stdin({"ssh_user": "root"}, "ssl", "apply", "--config", "/dev/null")
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()
