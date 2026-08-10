"""CLI commands must run without crashing."""

import subprocess

from . import helpers

PYTHON = helpers.PYTHON_BIN
PYTHON_ENV = helpers.PYTHON_ENV


def _run_no_input(*args):
    return subprocess.run(
        [PYTHON, "-m", "bonesinfra", *args],
        capture_output=True,
        text=True,
        timeout=10,
        env=PYTHON_ENV,
        check=False,
    )


def test_setup_apply_rejects_missing_host():
    result = _run_no_input("setup", "apply", "--config", "/dev/null", "--bonesremote-version", "0.7.3")
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()


def test_runtime_apply_rejects_missing_host():
    result = _run_no_input("runtime", "apply", "--config", "/dev/null")
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()


def test_ssl_apply_rejects_missing_domain_email():
    result = _run_no_input("ssl", "apply", "--config", "/dev/null")
    assert result.returncode == 3, f"Expected exit 3 for missing domain/email, got {result.returncode}"
    assert "ssl.domain" in result.stderr.lower()


def test_helpers_apply_rejects_missing_host():
    result = _run_no_input("helpers", "apply", "--config", "/dev/null")
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()


def test_framework_is_not_a_valid_subcommand():
    result = _run_no_input("framework", "apply", "--config", "/dev/null")
    assert result.returncode != 0, f"Expected non-zero exit for invalid 'framework' command, got {result.returncode}"
    assert "no such command" in result.stderr.lower()
