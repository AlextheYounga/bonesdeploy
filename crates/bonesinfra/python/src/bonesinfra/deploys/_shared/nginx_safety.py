from pyinfra.operations import files, server

from bonesinfra.domain.paths import ASSETS_DIR, SCRIPTS_DIR
from bonesinfra.infra.deploy_helpers import render


def install_default_deny_server(paths):
    # ponytail: self-signed is enough here because this server never serves
    # content; it only gives nginx a TLS default that can return 444.
    server.script_template(
        name="Ensure nginx default-deny SSL certificate exists",
        src=str(SCRIPTS_DIR / "ensure-default-deny-ssl.sh.j2"),
        cert=paths["nginx_default_deny_ssl_certificate"],
        key=paths["nginx_default_deny_ssl_certificate_key"],
        _sudo=True,
    )

    render(
        "Deploy nginx default-deny server",
        ASSETS_DIR / "nginx/default-deny.conf.j2",
        paths["nginx_default_deny_site_available"],
        mode="0644",
        paths=paths,
    )

    files.link(
        name="Enable nginx default-deny server",
        path=paths["nginx_default_deny_site_enabled"],
        target=paths["nginx_default_deny_site_available"],
        force=True,
        _sudo=True,
    )

    files.link(
        name="Disable Debian default nginx site",
        path=paths["nginx_default_site_enabled"],
        present=False,
        _sudo=True,
    )


def validate_config(name):
    server.script(
        name=name,
        src=str(SCRIPTS_DIR / "validate-nginx-safety.sh"),
        _sudo=True,
    )
