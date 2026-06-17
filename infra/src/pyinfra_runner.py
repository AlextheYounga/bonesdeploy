from __future__ import annotations

import sys
from typing import Any, Callable, Dict

from pyinfra.api import Config, Inventory, State
from pyinfra.api.connect import connect_all
from pyinfra.api.host import Host
from pyinfra.api.operations import run_ops


def run(
    *,
    hostname: str,
    ssh_user: str,
    ssh_port: int = 22,
    ssh_key: str | None = None,
    data: Dict[str, Any],
    deploy: Callable[[], None],
) -> None:
    config = Config(SSH_USER=ssh_user, SSH_PORT=ssh_port)
    if ssh_key:
        config.SSH_KEY = ssh_key
        config.SSH_STRICT_HOST_KEY_CHECKING = False

    host = Host(hostname, data=data)
    inventory = Inventory((host,), ())
    state = State(inventory, config)

    connect_all(state)

    deploy()

    run_ops(state)

    failed = getattr(state, "is_failed", False)
    if failed:
        print("pyinfra deploy completed with failures.", file=sys.stderr)
        sys.exit(1)
