from __future__ import annotations

import sys
from collections.abc import Callable
from contextlib import nullcontext
from types import ModuleType

from pyinfra.api import Config, Inventory, State
from pyinfra.api.connect import connect_all
from pyinfra.api.exceptions import PyinfraError
from pyinfra.api.operations import run_ops
from pyinfra.context import ctx_config, ctx_host, ctx_inventory, ctx_state

from bonesinfra.cli.hooks import load_custom_module
from bonesinfra.cli.output import (
    BonesDeployCallback,
    activity,
    print_banner,
    print_connected,
    print_done,
    print_target,
    setup_output,
    stop_live_output,
)
from bonesinfra.config.context import DeployContext


def run(
    *,
    ctx: DeployContext,
    config_path: str,
    deploy: Callable[[DeployContext, ModuleType | None], object | None],
    ssh_key: str | None = None,
    ssh_user_override: str | None = None,
    quiet: bool = False,
) -> object | None:
    if not quiet:
        setup_output()

    # Fail fast on custom.py syntax/import/shape errors before opening SSH.
    custom = load_custom_module(config_path)

    hostname = ctx.app.server.host
    ssh_user = ssh_user_override or ctx.app.server.ssh_user
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

    _show_target(hostname, ssh_user, quiet)
    _connect(state, quiet)
    result = _plan(ctx, custom, deploy, state, config, inventory, target_host, quiet)

    state.add_callback_handler(BonesDeployCallback())

    _run_operations(state, quiet)

    if not quiet:
        stop_live_output()
        print_done(success=True)
    return result


def _show_target(hostname: str, ssh_user: str, quiet: bool) -> None:
    if not quiet:
        print_banner()
        print_target(hostname, ssh_user)


def _connect(state: State, quiet: bool) -> None:
    try:
        with activity("connecting") if not quiet else nullcontext():
            connect_all(state)
    except PyinfraError:
        _fail(quiet)
    if not quiet:
        print_connected()


def _plan(ctx, custom, deploy, state, config, inventory, target_host, quiet):
    with (
        ctx_state.use(state),
        ctx_config.use(config),
        ctx_inventory.use(inventory),
        ctx_host.use(target_host),
        activity("planning deploy operations") if not quiet else nullcontext(),
    ):
        return deploy(ctx, custom)


def _run_operations(state: State, quiet: bool) -> None:
    try:
        run_ops(state)
    except PyinfraError:
        _fail(quiet)
    if state.failed_hosts:
        _fail(quiet)


def _fail(quiet: bool) -> None:
    if not quiet:
        stop_live_output()
        print_done(success=False)
    sys.exit(1)
