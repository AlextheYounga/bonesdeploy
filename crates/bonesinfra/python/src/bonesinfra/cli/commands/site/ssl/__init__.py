from shlex import quote

from pyinfra.operations import server

from bonesinfra.config.request import validate_domain, validate_email
from bonesinfra.pyinfra.operations import mkdir
from bonesinfra.services.linux import cloudflared, etckeeper
from bonesinfra.services.linux.nginx import router as nginx_router


def deploy_ssl(ctx):
    validate_domain(ctx.app.dns.domain)
    validate_email(ctx.app.dns.email)
    paths = ctx.paths_dict

    mkdir(
        name="Ensure ACME webroot exists",
        path=paths["acme_webroot"],
        mode="0755",
    )

    nginx_router.install_default_deny_server(paths)
    nginx_router.render_router_config(
        ctx, paths, ssl_enabled=False, stage="certbot challenge", validate=True, reload=True
    )
    obtain_certificate(ctx, paths)
    nginx_router.render_router_config(ctx, paths, ssl_enabled=True, stage="SSL enable", validate=True, reload=True)
    cloudflared.remove(ctx, paths)
    etckeeper.commit_changes("BonesInfra SSL provisioning")


def obtain_certificate(ctx, paths):
    server.shell(
        name="Obtain or renew certificate",
        commands=[
            "certbot certonly --non-interactive --agree-tos "
            f"--email {quote(ctx.app.dns.email)} "
            f"--webroot "
            f"-w {quote(paths['acme_webroot'])} "
            f"-d {quote(ctx.app.dns.domain)} "
            "--keep-until-expiring"
        ],
        _sudo=True,
    )
