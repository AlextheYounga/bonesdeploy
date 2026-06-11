import os

from pyinfra import host
from pyinfra.operations import files, server, systemd

here = os.path.dirname(__file__)

# Validate inputs
assert data.get("ssl_domain"), "ssl_domain is required"
assert data.get("ssl_email"), "ssl_email is required"

# Render nginx HTTP challenge config
files.template(
    name="Render nginx HTTP challenge config",
    src=os.path.join(here, "nginx/router.conf.j2"),
    dest=data["paths"]["nginx_site_available"],
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

server.shell(
    name="Obtain or renew certificate",
    commands=[
        "certbot certonly --non-interactive --agree-tos "
        f"--email {data['ssl_email']} "
        "--webroot "
        f"-w {data['paths']['current_web_root']} "
        f"-d {data['ssl_domain']} "
        "--keep-until-expiring"
    ],
    _sudo=True,
)

# Render nginx HTTPS config
files.template(
    name="Render nginx HTTPS config",
    src=os.path.join(here, "nginx/router.conf.j2"),
    dest=data["paths"]["nginx_site_available"],
    user="root",
    group="root",
    mode="0644",
    nginx_server_name=data["ssl_domain"],
    nginx_ssl_enabled=True,
    nginx_ssl_certificate_path=f"/etc/letsencrypt/live/{data['ssl_domain']}/fullchain.pem",
    nginx_ssl_certificate_key_path=f"/etc/letsencrypt/live/{data['ssl_domain']}/privkey.pem",
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
