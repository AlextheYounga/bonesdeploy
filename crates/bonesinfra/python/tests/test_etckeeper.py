"""Etckeeper change recording: package baseline, wrapper ordering, and record script behavior."""

import io
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from types import SimpleNamespace

import jinja2

from bonesinfra.cli import app
from bonesinfra.cli.commands.server.packages import BASE_SYSTEM_PACKAGES
from bonesinfra.services.linux import etckeeper

from .helpers import SRC_DIR, assert_not_contains, make_site_request

RECORD_SCRIPT = Path(etckeeper.RECORD_SCRIPT)
TEST_MESSAGE = "BonesInfra test changes"
GIT = shutil.which("git")
BASH = shutil.which("bash")

ETCKEEPER_SHIM = """#!/bin/sh
printf '%s\\n' "$*" >> "$SHIM_LOG"
if [ "$1" = "commit" ] && [ -n "$SHIM_COMMIT_FAILS" ]; then
    exit 1
fi
exit 0
"""


def test_server_baseline_installs_etckeeper():
    assert "etckeeper" in BASE_SYSTEM_PACKAGES


def test_commit_changes_after_runs_the_commit_after_the_deploy_plan(monkeypatch):
    calls = []
    monkeypatch.setattr(etckeeper, "commit_changes", lambda message: calls.append(("commit", message)))
    plan = etckeeper.commit_changes_after(lambda ctx: calls.append(("deploy", ctx)), TEST_MESSAGE)

    plan("site-context")

    assert calls == [("deploy", "site-context"), ("commit", TEST_MESSAGE)]


def test_runtime_apply_commits_changes_after_the_framework_deploy(monkeypatch):
    calls = []
    captured = {}
    monkeypatch.setattr(sys, "stdin", io.StringIO(json.dumps(make_site_request())))
    monkeypatch.setattr(
        app, "load_runtime", lambda _ctx: SimpleNamespace(deploy=lambda _ctx: calls.append("framework-deploy"))
    )
    monkeypatch.setattr(etckeeper, "commit_changes", lambda message: calls.append(("commit", message)))
    monkeypatch.setattr(app, "run", lambda **kwargs: captured.update(deploy=kwargs["deploy"]))

    app.runtime_apply_cmd(request_stdin=True)
    captured["deploy"]("site-context")

    assert calls == ["framework-deploy", ("commit", app.RUNTIME_CHANGE_MESSAGE)]


def test_read_only_and_patch_flows_do_not_record_changes():
    assert_not_contains((SRC_DIR / "bonesinfra/pyinfra/runner.py").read_text(), "etckeeper")
    assert_not_contains((SRC_DIR / "bonesinfra/manifest.py").read_text(), "etckeeper")
    for patch_module in sorted((SRC_DIR / "bonesinfra/patches").glob("*.py")):
        assert_not_contains(patch_module.read_text(), "etckeeper", msg=patch_module.name)


def _prepare_record_script(tmp_path: Path) -> Path:
    script = tmp_path / "etckeeper-record.sh"
    script.write_text(jinja2.Template(RECORD_SCRIPT.read_text()).render(message=TEST_MESSAGE))
    script.chmod(0o755)
    return script


def _prepare_git_etc(tmp_path: Path, *, dirty: bool) -> Path:
    etc = tmp_path / "etc"
    etc.mkdir()

    def git(*args):
        subprocess.run([GIT, *args], cwd=etc, check=True, capture_output=True, text=True)

    git("init")
    git("config", "user.email", "bones@example.test")
    git("config", "user.name", "Bones Test")
    (etc / "hosts").write_text("127.0.0.1 localhost\n")
    git("add", ".")
    git("-c", "commit.gpgsign=false", "commit", "-m", "initial")

    if dirty:
        (etc / "motd").write_text("provisioned\n")
    return etc


def _prepare_shim(tmp_path: Path) -> Path:
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    shim = bin_dir / "etckeeper"
    shim.write_text(ETCKEEPER_SHIM)
    shim.chmod(0o755)
    return bin_dir


def _run_record_script(script: Path, tmp_path: Path, bin_dir: Path | None, *, commit_fails: bool = False):
    env = {
        **os.environ,
        "PATH": f"{bin_dir}:/usr/bin:/bin" if bin_dir else "/usr/bin:/bin",
        "ETCKEEPER_DIR": str(tmp_path / "etc"),
        "SHIM_LOG": str(tmp_path / "shim.log"),
        "GIT_CONFIG_NOSYSTEM": "1",
    }
    if commit_fails:
        env["SHIM_COMMIT_FAILS"] = "1"
    return subprocess.run([BASH, str(script)], capture_output=True, text=True, env=env, check=False)


def _shim_log(tmp_path: Path) -> list[str]:
    log = tmp_path / "shim.log"
    return log.read_text().splitlines() if log.exists() else []


def test_record_script_commits_changes_when_etc_is_dirty(tmp_path):
    script = _prepare_record_script(tmp_path)
    _prepare_git_etc(tmp_path, dirty=True)
    bin_dir = _prepare_shim(tmp_path)

    result = _run_record_script(script, tmp_path, bin_dir)

    assert result.returncode == 0, result.stderr
    assert _shim_log(tmp_path) == ["init", f"commit {TEST_MESSAGE}"]


def test_record_script_does_not_commit_a_clean_etc(tmp_path):
    script = _prepare_record_script(tmp_path)
    _prepare_git_etc(tmp_path, dirty=False)
    bin_dir = _prepare_shim(tmp_path)

    result = _run_record_script(script, tmp_path, bin_dir)

    assert result.returncode == 0, result.stderr
    assert _shim_log(tmp_path) == ["init"]


def test_record_script_propagates_commit_failures(tmp_path):
    script = _prepare_record_script(tmp_path)
    _prepare_git_etc(tmp_path, dirty=True)
    bin_dir = _prepare_shim(tmp_path)

    result = _run_record_script(script, tmp_path, bin_dir, commit_fails=True)

    assert result.returncode != 0
    assert _shim_log(tmp_path) == ["init", f"commit {TEST_MESSAGE}"]


def test_record_script_fails_without_etckeeper(tmp_path):
    script = _prepare_record_script(tmp_path)
    _prepare_git_etc(tmp_path, dirty=True)

    result = _run_record_script(script, tmp_path, bin_dir=None)

    assert result.returncode != 0
    assert "etckeeper is not installed" in result.stderr


def test_record_script_fails_when_etc_is_not_an_etckeeper_repository(tmp_path):
    script = _prepare_record_script(tmp_path)
    etc = tmp_path / "etc"
    etc.mkdir()
    bin_dir = _prepare_shim(tmp_path)

    result = _run_record_script(script, tmp_path, bin_dir)

    assert result.returncode != 0
    assert "not a git-backed etckeeper repository" in result.stderr
