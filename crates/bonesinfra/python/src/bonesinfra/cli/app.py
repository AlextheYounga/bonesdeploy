import json
import sys

import typer

from bonesinfra.cli.commands.helpers import deploy_helpers
from bonesinfra.cli.commands.runtime import deploy_runtime
from bonesinfra.cli.commands.services import deploy_services
from bonesinfra.cli.commands.setup import deploy_setup
from bonesinfra.cli.commands.ssl import deploy_ssl
from bonesinfra.config.context import DeployContext
from bonesinfra.frameworks import list_frameworks
from bonesinfra.manifest import inspect_for_runner, render
from bonesinfra.pyinfra.runner import run

app = typer.Typer()
runtime_app = typer.Typer()
setup_app = typer.Typer()
ssl_app = typer.Typer()
helpers_app = typer.Typer()
services_app = typer.Typer()
manifest_app = typer.Typer()
app.add_typer(runtime_app, name="runtime", help="Runtime operations")
app.add_typer(setup_app, name="setup", help="Setup operations")
app.add_typer(ssl_app, name="ssl", help="SSL operations")
app.add_typer(helpers_app, name="helpers", help="Helper tool operations")
app.add_typer(services_app, name="services", help="Service operations")
app.add_typer(manifest_app, name="manifest", help="Manifest inspection")


def _validate_host(ctx: DeployContext) -> None:
    if not ctx.app.server.host:
        print("Error: missing host in bones.toml", file=sys.stderr)
        sys.exit(3)


@runtime_app.command("list")
def runtime_list():
    print(json.dumps(list_frameworks()))


@runtime_app.command("apply")
def runtime_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
):
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    run(ctx=ctx, config_path=config, deploy=deploy_runtime)


@setup_app.command("apply")
def setup_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
    bonesremote_version: str = typer.Option(..., "--bonesremote-version", help="Release version to install"),
):
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    run(
        ctx=ctx,
        config_path=config,
        deploy=lambda ctx, custom: deploy_setup(ctx, custom, bonesremote_version),
    )


@ssl_app.command("apply")
def ssl_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
):
    ctx = DeployContext.from_files(config)
    if not ctx.app.dns.domain or not ctx.app.dns.email:
        print("Error: ssl.domain and ssl.email are required in bones.toml", file=sys.stderr)
        sys.exit(3)
    _validate_host(ctx)
    run(ctx=ctx, config_path=config, deploy=deploy_ssl)


@helpers_app.command("apply")
def helpers_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
):
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    run(ctx=ctx, config_path=config, deploy=deploy_helpers)


@services_app.command("apply")
def services_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
):
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    run(ctx=ctx, config_path=config, deploy=deploy_services)


@manifest_app.command("show")
def manifest_show_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
    output_format: str = typer.Option("text", "--format", help="Output format: text or json"),
):
    if output_format not in {"text", "json"}:
        raise typer.BadParameter("must be text or json", param_hint="--format")
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    data = run(ctx=ctx, config_path=config, deploy=inspect_for_runner, quiet=True)
    if not isinstance(data, dict):
        raise TypeError("manifest inspection returned no report")
    print(render(data, output_format))
