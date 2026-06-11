import os

from pyinfra import host
from pyinfra.operations import files, server, systemd
from utils import unflatten


here = os.path.dirname(__file__)
DEPLOY_DATA = unflatten(host.data.dict())
PATHS = DEPLOY_DATA.get("paths", {})

# Validate inputs
assert DEPLOY_DATA.get("ssl_domain"), "ssl_domain is required"
assert DEPLOY_DATA.get("ssl_email"), "ssl_email is required"

# Render nginx HTTP challenge config
files.template(
    name="Render nginx HTTP challenge config",
    src=os.path.join(here, "nginx/router.conf.j2"),
    dest=PATHS["nginx_site_available"],
    user="root",
    group="root",
    mode="0644",
    nginx_server_name=DEPLOY_DATA["ssl_domain"],
    nginx_ssl_enabled=False,
    **DEPLOY_DATA,
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
        f"--email {DEPLOY_DATA["ssl_email"]} "
        "--webroot "
        f"-w {PATHS['current_web_root']} "
        f"-d {DEPLOY_DATA["ssl_domain"]} "
        "--keep-until-expiring"
    ],
    _sudo=True,
)

# Render nginx HTTPS config
files.template(
    name="Render nginx HTTPS config",
    src=os.path.join(here, "nginx/router.conf.j2"),
    dest=PATHS["nginx_site_available"],
    user="root",
    group="root",
    mode="0644",
    nginx_server_name=DEPLOY_DATA["ssl_domain"],
    nginx_ssl_enabled=True,
    nginx_ssl_certificate_path=f"/etc/letsencrypt/live/{DEPLOY_DATA["ssl_domain"]}/fullchain.pem",
    nginx_ssl_certificate_key_path=f"/etc/letsencrypt/live/{DEPLOY_DATA["ssl_domain"]}/privkey.pem",
    **DEPLOY_DATA,
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
