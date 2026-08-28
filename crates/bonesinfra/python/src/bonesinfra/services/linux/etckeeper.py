"""Etckeeper change recording for /etc across mutating provisioning flows."""

from collections.abc import Callable
from typing import Any

from pyinfra.operations import server

from bonesinfra.config.paths import SCRIPTS_DIR

RECORD_SCRIPT = SCRIPTS_DIR / "etckeeper-record.sh.j2"


def initialize():
    """Ensure /etc is an etckeeper-managed repository (idempotent)."""
    server.shell(
        name="Initialize etckeeper in /etc",
        commands=["etckeeper", "init"],
        _sudo=True,
    )


def commit_changes(message: str):
    """Record /etc changes as the final operation of a provisioning flow."""
    server.script_template(
        name="Record /etc changes with etckeeper",
        src=str(RECORD_SCRIPT),
        message=message,
        _sudo=True,
    )


def commit_changes_after(deploy: Callable[[Any], Any], message: str) -> Callable[[Any], Any]:
    """Wrap a deploy plan so /etc changes are committed after its operations succeed."""

    def plan(ctx: Any) -> Any:
        result = deploy(ctx)
        commit_changes(message)
        return result

    return plan
