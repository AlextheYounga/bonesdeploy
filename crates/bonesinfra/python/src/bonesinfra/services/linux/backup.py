"""Scheduled Borg backups of the site's shared data."""

from io import StringIO
from pathlib import Path
from shlex import quote

from pyinfra.operations import apt, files, server

from bonesinfra.config.paths import ASSETS_DIR, BACKUPS_ROOT
from bonesinfra.pyinfra.operations import mkdir, render


def provision(ctx, paths):
    """Install Borg, create the encrypted repository, and install the cron schedule.

    Sites without a configured passphrase keep their pre-backup behavior.
    """
    backup = ctx.backup
    if not backup.configured:
        return
    apt.packages(
        name="Install Borg",
        packages=["borgbackup"],
        present=True,
        update=True,
        cache_time=3600,
        _sudo=True,
    )
    mkdir(name="Ensure backup repository root exists", path=BACKUPS_ROOT, mode="0700")
    mkdir(
        name="Ensure BonesRemote site state directory exists",
        path=str(Path(paths["backup_passphrase_file"]).parent),
        mode="0700",
    )
    files.put(
        name="Install root-only Borg passphrase file",
        src=StringIO(backup.passphrase),
        dest=paths["backup_passphrase_file"],
        user="root",
        group="root",
        mode="0600",
        _sudo=True,
    )
    server.shell(
        name="Create encrypted Borg repository",
        commands=[_init_command(paths)],
        _sudo=True,
    )
    render(
        name="Install scheduled backup cron entry",
        src=str(ASSETS_DIR / "cron/backup.cron.j2"),
        dest=paths["backup_cron_file"],
        user="root",
        group="root",
        mode="0644",
        project_name=ctx.app.project_name,
        backup_schedule=backup.schedule,
        backup_keep_days=backup.retention_days,
        bonesremote_path=paths["bonesremote_global_link"],
    )


def _init_command(paths) -> str:
    """Create the repokey-blake2 repository once; the passphrase never reaches argv."""
    repository = quote(paths["backup_repository"])
    passphrase_file = quote(paths["backup_passphrase_file"])
    return (
        f"if [ ! -d {repository} ]; then "
        f'BORG_PASSPHRASE="$(cat {passphrase_file})" '
        f"borg init --encryption repokey-blake2 {repository}; fi"
    )
