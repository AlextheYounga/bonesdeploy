import os

from pyinfra import host
from pyinfra.operations import files, server, systemd
from src.utils import unflatten


def deploy():
    data = unflatten(host.data.dict())
    paths = data.get("paths", {})
    here = os.path.dirname(__file__)

    assert data.get("ssl_domain"), "ssl_domain is required"
    assert data.get("ssl_email"), "ssl_email is required"

    render_http_challenge_config(data, paths, here)
    obtain_certificate(data, paths)
    render_https_config(data, paths, here)


def render_http_challenge_config(data, paths, here):
    files.template(
        name="Render nginx HTTP challenge config",
        src=os.path.join(here, "assets/nginx/router.conf.j2"),
        dest=paths["nginx_site_available"],
        user="root",
        group="root",
        mode="0644",
        nginx_server_name=data["ssl_domain"],
        nginx_ssl_enabled=False,
        **data,
        _sudo=True,
    )

    server.shell(
        name="Validate nginx configuration before certbot",
        commands=["nginx -t"],
        _sudo=True,
    )

    systemd.service(
        name="Reload nginx before certbot challenge",
        service="nginx",
        reloaded=True,
        _sudo=True,
    )


def obtain_certificate(data, paths):
    server.shell(
        name="Obtain or renew certificate",
        commands=[
            "certbot certonly --non-interactive --agree-tos "
            f"--email {data['ssl_email']} "
            "--webroot "
            f"-w {paths['current_web_root']} "
            f"-d {data['ssl_domain']} "
            "--keep-until-expiring"
        ],
        _sudo=True,
    )


def render_https_config(data, paths, here):
    files.template(
        name="Render nginx HTTPS config",
        src=os.path.join(here, "assets/nginx/router.conf.j2"),
        dest=paths["nginx_site_available"],
        user="root",
        group="root",
        mode="0644",
        nginx_server_name=data["ssl_domain"],
        nginx_ssl_enabled=True,
        **data,
        _sudo=True,
    )

    server.shell(
        name="Validate nginx configuration after SSL enable",
        commands=["nginx -t"],
        _sudo=True,
    )

    systemd.service(
        name="Reload nginx with SSL configuration",
        service="nginx",
        reloaded=True,
        _sudo=True,
    )

