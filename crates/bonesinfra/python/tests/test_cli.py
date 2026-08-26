"""CLI commands must run without crashing."""

import json
import subprocess

from . import helpers

PYTHON = helpers.PYTHON_BIN
PYTHON_ENV = helpers.PYTHON_ENV


def _run_no_input(*args, input_text=None):
    return subprocess.run(
        [PYTHON, "-m", "bonesinfra", *args],
        capture_output=True,
        text=True,
        input=json.dumps(input_text) if isinstance(input_text, dict) else input_text,
        timeout=10,
        env=PYTHON_ENV,
        check=False,
    )


def test_server_apply_rejects_missing_host():
    result = _run_no_input(
        "server", "apply", "--request-stdin", "--bonesremote-version", "0.7.3", input_text={"server": {}}
    )
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()


def test_setup_apply_command_is_removed():
    result = _run_no_input(
        "setup", "apply", "--request-stdin", "--bonesremote-version", "0.7.3", input_text={"server": {}}
    )
    assert result.returncode != 0, f"Expected non-zero exit for removed setup command, got {result.returncode}"
    assert "no such command" in result.stderr.lower()


def test_runtime_apply_rejects_missing_host():
    result = _run_no_input(
        "runtime", "apply", "--request-stdin", input_text={"server": {}, "site": {"project_name": "demo"}}
    )
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()


def test_ssl_apply_rejects_missing_domain_email():
    result = _run_no_input(
        "ssl", "apply", "--request-stdin", input_text={"server": {}, "site": {"project_name": "demo"}}
    )
    assert result.returncode == 3, f"Expected exit 3 for missing domain/email, got {result.returncode}"
    assert "domain" in result.stderr.lower()
    assert "email" in result.stderr.lower()


def test_helpers_apply_rejects_missing_host():
    result = _run_no_input(
        "helpers", "apply", "--request-stdin", input_text={"server": {}, "site": {"project_name": "demo"}}
    )
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()


def test_framework_is_not_a_valid_subcommand():
    result = _run_no_input("framework", "apply", "--request-stdin", input_text={"server": {}})
    assert result.returncode != 0, f"Expected non-zero exit for invalid 'framework' command, got {result.returncode}"
    assert "no such command" in result.stderr.lower()


def test_commands_expose_request_stdin_instead_of_env_file():
    result = _run_no_input("runtime", "apply", "--help")
    assert result.returncode == 0
    assert "--request-stdin" in result.stdout
    assert "--env-file" not in result.stdout
