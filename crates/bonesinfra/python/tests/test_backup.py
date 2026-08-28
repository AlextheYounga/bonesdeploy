"""Backup request parsing, provisioning operations, and cron rendering."""

import io
from pathlib import Path
from types import SimpleNamespace

import pytest
from jinja2 import Environment

from bonesinfra.config.context import BackupConfig
from bonesinfra.config.request import parse_site
from bonesinfra.services.linux import backup

PASSPHRASE = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"  # noqa: S105 - synthetic fixture


def parse_backup(overrides):
    backup_section = {"schedule": "0 0 * * *", "retention_days": 30, "passphrase": ""}
    backup_section.update(overrides)
    body = {
        "server": {"host": "example.com", "ssh_user": "root", "port": "22"},
        "site": {"project_name": "lawsnipe", "backup": backup_section},
    }
    return parse_site(body).backup


def test_backup_section_parses_into_typed_configuration():
    backup = parse_backup({"schedule": "15 2 * * *", "retention_days": 21, "passphrase": PASSPHRASE})

    assert backup.schedule == "15 2 * * *"
    assert backup.retention_days == 21
    assert backup.passphrase == PASSPHRASE
    assert backup.configured is True


def test_empty_passphrase_marks_backups_unconfigured():
    assert parse_backup({}).configured is False


@pytest.mark.parametrize(
    "backup_value",
    [
        None,
        "not-an-object",
        {"schedule": "0 0 * *", "retention_days": 30, "passphrase": ""},
        {"schedule": "0 0 * * *; rm -rf /", "retention_days": 30, "passphrase": ""},
        {"schedule": "0 0 * * * %", "retention_days": 30, "passphrase": ""},
        {"schedule": "0 0 * * *", "retention_days": 0, "passphrase": ""},
        {"schedule": "0 0 * * *", "retention_days": "30", "passphrase": ""},
        {"schedule": "0 0 * * *", "retention_days": True, "passphrase": ""},
        {"schedule": "0 0 * * *", "retention_days": 30, "passphrase": "", "intruder": 1},
        {"schedule": "0 0 * * *", "retention_days": 30, "passphrase": " leading"},
        {"schedule": "0 0 * * *", "retention_days": 30, "passphrase": "new\nline"},
        {"schedule": "0 0 * * *", "retention_days": 30, "passphrase": "brace{}"},
        {"schedule": "0 0 * * *", "retention_days": 30, "passphrase": "percent%"},
        {"schedule": "0 0 * * *", "retention_days": 30, "passphrase": "stéphane"},
    ],
)
def test_invalid_backup_sections_are_rejected(backup_value):
    site = {"project_name": "lawsnipe"}
    if backup_value is not None:
        site["backup"] = backup_value
    body = {"server": {"host": "example.com", "ssh_user": "root", "port": "22"}, "site": site}
    with pytest.raises((ValueError, TypeError)):
        parse_site(body)


def test_provisioning_is_a_no_operation_for_unconfigured_backups(monkeypatch):
    def fail(**_kwargs):
        raise AssertionError("no operation should run for unconfigured backups")

    monkeypatch.setattr(backup.apt, "packages", fail)
    monkeypatch.setattr(backup.files, "put", fail)
    monkeypatch.setattr(backup.server, "shell", fail)
    monkeypatch.setattr(backup, "mkdir", fail)
    monkeypatch.setattr(backup, "render", fail)

    backup.provision(_ctx(BackupConfig("0 0 * * *", 30, "")), _paths())


def test_provisioning_installs_borg_passphrase_repository_and_cron(monkeypatch):
    calls = []
    monkeypatch.setattr(backup.apt, "packages", lambda **kwargs: calls.append(("packages", kwargs)))
    monkeypatch.setattr(backup, "mkdir", lambda **kwargs: calls.append(("mkdir", kwargs)))
    monkeypatch.setattr(backup.files, "put", lambda **kwargs: calls.append(("put", kwargs)))
    monkeypatch.setattr(backup.server, "shell", lambda **kwargs: calls.append(("shell", kwargs)))
    monkeypatch.setattr(backup, "render", lambda **kwargs: calls.append(("render", kwargs)))

    backup.provision(_ctx(BackupConfig("15 2 * * *", 21, PASSPHRASE)), _paths())

    assert [operation for operation, _kwargs in calls] == ["packages", "mkdir", "mkdir", "put", "shell", "render"]

    package_kwargs = calls[0][1]
    assert package_kwargs["packages"] == ["borgbackup"]

    passphrase_kwargs = calls[3][1]
    assert passphrase_kwargs["dest"] == "/root/.config/bonesremote/sites/atlas/.borg_passphrase"
    assert passphrase_kwargs["mode"] == "0600"
    assert isinstance(passphrase_kwargs["src"], io.StringIO)
    assert passphrase_kwargs["src"].getvalue() == PASSPHRASE

    shell_command = calls[4][1]["commands"][0]
    assert "/var/lib/bonesdeploy/backups/atlas.borg" in shell_command
    assert "repokey-blake2" in shell_command
    assert "cat /root/.config/bonesremote/sites/atlas/.borg_passphrase" in shell_command
    assert PASSPHRASE not in shell_command, "the passphrase must never reach a command line"

    render_kwargs = calls[5][1]
    assert render_kwargs["dest"] == "/etc/cron.d/bonesdeploy-atlas-backup"
    assert render_kwargs["mode"] == "0644"
    assert render_kwargs["backup_schedule"] == "15 2 * * *"
    assert render_kwargs["backup_keep_days"] == 21
    assert render_kwargs["bonesremote_path"] == "/usr/local/bin/bonesremote"


def test_cron_template_renders_the_schedule_without_the_passphrase():
    template_path = Path(backup.ASSETS_DIR / "cron/backup.cron.j2")
    # pyinfra renders templates with keep_trailing_newline=True (api/util.py);
    # autoescape stays off like pyinfra's environment because this renders shell text.
    template = Environment(keep_trailing_newline=True, autoescape=False).from_string(template_path.read_text())  # noqa: S701
    rendered = template.render(
        project_name="atlas",
        backup_schedule="15 2 * * *",
        backup_keep_days=21,
        bonesremote_path="/usr/local/bin/bonesremote",
    )

    assert "15 2 * * * root /usr/local/bin/bonesremote backup run --site atlas --keep-days 21" in rendered
    assert "systemd-cat -t bonesdeploy-backup" in rendered
    assert 'MAILTO=""' in rendered
    assert rendered.endswith("\n"), "cron files must end with a newline"
    assert PASSPHRASE not in rendered


def _ctx(backup_config):
    return SimpleNamespace(app=SimpleNamespace(project_name="atlas"), backup=backup_config)


def _paths():
    return {
        "backup_repository": "/var/lib/bonesdeploy/backups/atlas.borg",
        "backup_passphrase_file": "/root/.config/bonesremote/sites/atlas/.borg_passphrase",
        "backup_cron_file": "/etc/cron.d/bonesdeploy-atlas-backup",
        "bonesremote_global_link": "/usr/local/bin/bonesremote",
    }
