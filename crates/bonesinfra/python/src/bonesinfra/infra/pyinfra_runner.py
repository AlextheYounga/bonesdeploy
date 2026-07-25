from __future__ import annotations

import sys
from collections.abc import Callable
from types import ModuleType

from pyinfra.api import Config, Inventory, State
from pyinfra.api.connect import connect_all
from pyinfra.api.exceptions import PyinfraError
from pyinfra.api.operations import run_ops
from pyinfra.context import ctx_config, ctx_host, ctx_inventory, ctx_state

from bonesinfra.domain.context import DeployContext
from bonesinfra.domain.custom import load_custom_module
from bonesinfra.infra.output import (
    BonesDeployCallback,
    activity,
    print_banner,
    print_connected,
    print_done,
    print_target,
    setup_output,
    stop_live_output,
)


def run(
    *,
    ctx: DeployContext,
    config_path: str,
    deploy: Callable[[DeployContext, ModuleType | None], None],
    ssh_key: str | None = None,
) -> None:
    setup_output()

    # Fail fast on custom.py syntax/import/shape errors before opening SSH.
    custom = load_custom_module(config_path)

    hostname = ctx.app.server.host
    ssh_user = ctx.app.server.ssh_user
    ssh_port = int(ctx.app.server.port)

    host_data: dict[str, object] = {
        "ssh_user": ssh_user,
        "ssh_port": ssh_port,
    }
    if ssh_key:
        host_data["ssh_key"] = ssh_key

    config = Config()

    inventory = Inventory(([(hostname, host_data)], {}))
    state = State(inventory, config)
    target_host = next(iter(inventory))

    print_banner()
    print_target(hostname, ssh_user)

    try:
        with activity("connecting"):
            connect_all(state)
    except PyinfraError:
        print_done(success=False)
        sys.exit(1)

    print_connected()

    with (
        ctx_state.use(state),
        ctx_config.use(config),
        ctx_inventory.use(inventory),
        ctx_host.use(target_host),
        activity("planning deploy operations"),
    ):
        deploy(ctx, custom)

    state.add_callback_handler(BonesDeployCallback())

    try:
        run_ops(state)
    except PyinfraError:
        stop_live_output()
        print_done(success=False)
        sys.exit(1)

    if state.failed_hosts:
        stop_live_output()
        print_done(success=False)
        sys.exit(1)

    stop_live_output()
    print_done(success=True)
