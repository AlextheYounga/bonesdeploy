from shlex import quote

from pyinfra.operations import apt, files, server, systemd

from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.services.linux import systemd as service


def install():
    """Install cloudflared from Cloudflare's supported package repository."""
    files.directory(
        name="Ensure APT keyring directory exists",
        path="/etc/apt/keyrings",
        user="root",
        group="root",
        mode="0755",
        _sudo=True,
    )
    files.download(
        name="Install Cloudflare package signing key",
        src="https://pkg.cloudflare.com/cloudflare-main.gpg",
        dest="/etc/apt/keyrings/cloudflare-main.gpg",
        user="root",
        group="root",
        mode="0644",
        _sudo=True,
    )
    files.template(
        name="Install Cloudflare package source",
        src=str(ASSETS_DIR / "apt/cloudflared.list.j2"),
        dest="/etc/apt/sources.list.d/cloudflared.list",
        user="root",
        group="root",
        mode="0644",
        _sudo=True,
    )
    apt.packages(name="Install cloudflared", packages=["cloudflared"], present=True, update=True, _sudo=True)


def setup(ctx, paths):
    files.template(
        name="Deploy project quick tunnel service",
        src=str(ASSETS_DIR / "systemd/cloudflared.service.j2"),
        dest=paths["systemd_cloudflared_service"],
        user="root",
        group="root",
        mode="0644",
        **template_data(ctx, paths=paths),
        _sudo=True,
    )
    service.register_service(ctx, paths=paths, name="cloudflared")
    systemd.daemon_reload(name="Reload systemd after quick tunnel change", _sudo=True)


def start(ctx):
    service.enable_and_start(ctx, "cloudflared")


def remove(ctx, paths):
    service_name = f"{ctx.app.project_name}-cloudflared.service"
    systemd.service(name="Stop project quick tunnel", service=service_name, running=False, enabled=False, _sudo=True)
    server.shell(
        name="Remove project quick tunnel",
        commands=[
            f"rm -f -- {quote(paths['systemd_site_target_requires'] + '/' + service_name)}",
            f"rm -f -- {quote(paths['systemd_cloudflared_service'])}",
        ],
        _sudo=True,
    )
    systemd.daemon_reload(name="Reload systemd after quick tunnel removal", _sudo=True)
