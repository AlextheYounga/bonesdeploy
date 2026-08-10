import sys

import typer

from bonesinfra.cli.commands.helpers import deploy_helpers
from bonesinfra.cli.commands.services import deploy_services
from bonesinfra.cli.commands.setup import deploy_setup
from bonesinfra.cli.commands.ssl import deploy_ssl
from bonesinfra.config.context import DeployContext
from bonesinfra.manifest import inspect_for_runner, render
from bonesinfra.patches import apply_local, apply_remote
from bonesinfra.project import load_manifest, load_runtime
from bonesinfra.pyinfra.runner import run

app = typer.Typer()
runtime_app = typer.Typer()
setup_app = typer.Typer()
ssl_app = typer.Typer()
helpers_app = typer.Typer()
services_app = typer.Typer()
manifest_app = typer.Typer()
patches_app = typer.Typer()
app.add_typer(runtime_app, name="runtime", help="Runtime operations")
app.add_typer(setup_app, name="setup", help="Setup operations")
app.add_typer(ssl_app, name="ssl", help="SSL operations")
app.add_typer(helpers_app, name="helpers", help="Helper tool operations")
app.add_typer(services_app, name="services", help="Service operations")
app.add_typer(manifest_app, name="manifest", help="Manifest inspection")
app.add_typer(patches_app, name="patches", help="Update migrations")


def _validate_host(ctx: DeployContext) -> None:
    if not ctx.app.server.host:
        print("Error: missing host in bones.toml", file=sys.stderr)
        sys.exit(3)


@runtime_app.command("apply")
def runtime_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
):
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    project_runtime = load_runtime(config)
    run(ctx=ctx, deploy=project_runtime.deploy)


@setup_app.command("apply")
def setup_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
    bonesremote_version: str = typer.Option(..., "--bonesremote-version", help="Release version to install"),
):
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    run(
        ctx=ctx,
        deploy=lambda ctx: deploy_setup(ctx, bonesremote_version),
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
    run(ctx=ctx, deploy=deploy_ssl)


@helpers_app.command("apply")
def helpers_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
):
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    run(ctx=ctx, deploy=deploy_helpers)


@services_app.command("apply")
def services_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
):
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    run(ctx=ctx, deploy=deploy_services)


@manifest_app.command("show")
def manifest_show_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
    output_format: str = typer.Option("text", "--format", help="Output format: text or json"),
):
    if output_format not in {"text", "json"}:
        raise typer.BadParameter("must be text or json", param_hint="--format")
    ctx = DeployContext.from_files(config)
    _validate_host(ctx)
    project_manifest = load_manifest(config)
    data = run(ctx=ctx, deploy=lambda current_ctx: inspect_for_runner(current_ctx, project_manifest), quiet=True)
    if not isinstance(data, dict):
        raise TypeError("manifest inspection returned no report")
    print(render(data, output_format))


@patches_app.command("apply")
def patches_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
    target_version: str = typer.Option(..., "--target-version", help="Version being updated to"),
    scope: str = typer.Option(..., "--scope", help="Patch scope: local or remote"),
):
    if scope not in {"local", "remote"}:
        raise typer.BadParameter("must be local or remote", param_hint="--scope")
    ctx = DeployContext.from_files(config)
    if scope == "local":
        apply_local(ctx, target_version, config)
        return
    _validate_host(ctx)
    run(
        ctx=ctx,
        deploy=lambda patch_ctx: apply_remote(patch_ctx, target_version),
        ssh_user_override="root",
    )
