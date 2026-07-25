from types import ModuleType

from pyinfra.operations import server, systemd

from bonesinfra.deploys._shared import nginx_safety
from bonesinfra.domain.context import template_data
from bonesinfra.domain.custom import call_hook
from bonesinfra.domain.paths import ASSETS_DIR
from bonesinfra.infra.deploy_helpers import letsencrypt_cert_paths, mkdir, render


def deploy_ssl(ctx, custom: ModuleType | None = None):
    paths = ctx.paths_dict

    mkdir(
        name="Ensure ACME webroot exists",
        path=paths["acme_webroot"],
        mode="0755",
    )

    nginx_safety.install_default_deny_server(paths)
    _render_router_config(ctx, paths, ssl_enabled=False, stage="certbot challenge")
    obtain_certificate(ctx, paths)
    _render_router_config(ctx, paths, ssl_enabled=True, stage="SSL enable")

    call_hook(custom, "after_ssl", ctx)


def _render_router_config(ctx, paths, ssl_enabled, stage):
    nginx_server_name = ctx.app.dns.domain

    extra = {
        "nginx_server_name": nginx_server_name,
        "nginx_ssl_enabled": ssl_enabled,
    }
    if ssl_enabled:
        cert_path, key_path = letsencrypt_cert_paths(nginx_server_name)
        extra["nginx_ssl_certificate_path"] = cert_path
        extra["nginx_ssl_certificate_key_path"] = key_path

    render(
        f"Render nginx config ({stage})",
        ASSETS_DIR / "nginx/router.conf.j2",
        paths["nginx_site_available"],
        mode="0644",
        **extra,
        **template_data(ctx, paths=paths),
    )

    nginx_safety.validate_config(f"Validate nginx configuration ({stage})")

    systemd.service(
        name=f"Reload nginx ({stage})",
        service="nginx",
        reloaded=True,
        _sudo=True,
    )


def obtain_certificate(ctx, paths):
    server.shell(
        name="Obtain or renew certificate",
        commands=[
            "certbot certonly --non-interactive --agree-tos "
            f"--email {ctx.app.dns.email} "
            f"--webroot "
            f"-w {paths['acme_webroot']} "
            f"-d {ctx.app.dns.domain} "
            "--keep-until-expiring"
        ],
        _sudo=True,
    )
