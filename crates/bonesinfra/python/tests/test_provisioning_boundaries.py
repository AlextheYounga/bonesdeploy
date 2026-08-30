from types import SimpleNamespace

from bonesinfra.cli.commands import server, site
from bonesinfra.cli.commands.server import helpers as server_helpers
from bonesinfra.cli.commands.site import services as site_services, ssl as site_ssl


def test_server_setup_runs_only_server_operations(monkeypatch):
    calls = []
    monkeypatch.setattr(server.packages, "install_system", lambda *_: calls.append("packages"))
    monkeypatch.setattr(server.etckeeper, "initialize", lambda: calls.append("etckeeper-init"))
    monkeypatch.setattr(server.apparmor, "ensure_service", lambda: calls.append("apparmor"))
    monkeypatch.setattr(server.disable_algif_aead, "configure", lambda: calls.append("hardening"))
    monkeypatch.setattr(server.image_store, "ensure_shared_store", lambda: calls.append("image-store"))
    monkeypatch.setattr(server.image_store, "seed_base_image", lambda: calls.append("base-image"))
    monkeypatch.setattr(server.firewall, "configure", lambda *_: calls.append("firewall"))
    monkeypatch.setattr(server.fail2ban, "configure", lambda *_: calls.append("fail2ban"))
    monkeypatch.setattr(server.unattended_upgrades, "configure", lambda: calls.append("upgrades"))
    monkeypatch.setattr(server.users, "ensure_deploy_user", lambda *_: calls.append("deploy-user"))
    monkeypatch.setattr(server.sshd, "disable_root_password_login", lambda: calls.append("root-login-hardening"))
    monkeypatch.setattr(server, "_ensure_bonesremote_roots", lambda: calls.append("bonesremote-roots"))
    monkeypatch.setattr(server.bonesremote, "install", lambda *_: calls.append("bonesremote"))
    monkeypatch.setattr(server.sudoers, "install", lambda: calls.append("sudoers"))
    monkeypatch.setattr(server.etckeeper, "commit_changes", lambda *_: calls.append("etckeeper-commit"))

    server.deploy_server_setup(SimpleNamespace(), "1.0.0")

    assert calls == [
        "packages",
        "etckeeper-init",
        "apparmor",
        "hardening",
        "image-store",
        "base-image",
        "firewall",
        "fail2ban",
        "upgrades",
        "deploy-user",
        "root-login-hardening",
        "bonesremote-roots",
        "bonesremote",
        "sudoers",
        "etckeeper-commit",
    ]


def test_site_setup_runs_only_site_base_operations(monkeypatch):
    calls = []
    ctx = SimpleNamespace(paths_dict={})
    monkeypatch.setattr(site.users, "ensure_users_and_groups", lambda *_: calls.append("identities"))
    monkeypatch.setattr(site.directories, "setup_repo_and_project", lambda *_: calls.append("directories"))
    monkeypatch.setattr(site.placeholder, "seed", lambda *_: calls.append("placeholder"))
    monkeypatch.setattr(site.backup, "provision", lambda *_: calls.append("backup"))
    monkeypatch.setattr(site.etckeeper, "commit_changes", lambda *_: calls.append("etckeeper-commit"))

    site.deploy_site_setup(ctx)

    assert calls == ["identities", "directories", "placeholder", "backup", "etckeeper-commit"]


def test_service_provisioning_records_changes_after_provisioning(monkeypatch):
    calls = []
    ctx = SimpleNamespace(services=SimpleNamespace(services=("postgres", "valkey")))
    monkeypatch.setattr(
        site_services,
        "get_service",
        lambda name: SimpleNamespace(provision=lambda _ctx: calls.append(f"provision-{name}")),
    )
    monkeypatch.setattr(site_services.etckeeper, "commit_changes", lambda *_: calls.append("etckeeper-commit"))

    site_services.deploy_services(ctx)

    assert calls == ["provision-postgres", "provision-valkey", "etckeeper-commit"]


def test_ssl_provisioning_records_changes_after_ssl_operations(monkeypatch):
    calls = []
    ctx = SimpleNamespace(
        paths_dict={"acme_webroot": "/var/www/app"},
        app=SimpleNamespace(dns=SimpleNamespace(domain="app.example.test", email="ops@example.test")),
    )
    monkeypatch.setattr(site_ssl, "mkdir", lambda **_kwargs: calls.append("webroot"))
    monkeypatch.setattr(site_ssl.nginx_router, "install_default_deny_server", lambda *_: calls.append("default-deny"))
    monkeypatch.setattr(
        site_ssl.nginx_router,
        "render_router_config",
        lambda *_args, **kwargs: calls.append(f"router:{kwargs['stage']}"),
    )
    monkeypatch.setattr(site_ssl, "obtain_certificate", lambda *_: calls.append("certbot"))
    monkeypatch.setattr(site_ssl.cloudflared, "remove", lambda *_: calls.append("tunnel-remove"))
    monkeypatch.setattr(site_ssl.etckeeper, "commit_changes", lambda *_: calls.append("etckeeper-commit"))

    site_ssl.deploy_ssl(ctx)

    assert calls == [
        "webroot",
        "default-deny",
        "router:certbot challenge",
        "certbot",
        "router:SSL enable",
        "tunnel-remove",
        "etckeeper-commit",
    ]


def test_helper_provisioning_records_changes_after_helper_installation(monkeypatch):
    calls = []
    monkeypatch.setattr(server_helpers.packages, "install_helper_apt_packages", lambda *_: calls.append("packages"))
    monkeypatch.setattr(server_helpers.packages, "install_debian_command_aliases", lambda: calls.append("aliases"))
    monkeypatch.setattr(server_helpers.starship, "install", lambda: calls.append("starship"))
    monkeypatch.setattr(server_helpers.neovim, "install", lambda: calls.append("neovim"))
    monkeypatch.setattr(server_helpers.rainfrog, "install", lambda: calls.append("rainfrog"))
    monkeypatch.setattr(server_helpers.etckeeper, "commit_changes", lambda *_: calls.append("etckeeper-commit"))

    server_helpers.deploy_helpers(SimpleNamespace())

    assert calls == ["packages", "aliases", "starship", "neovim", "rainfrog", "etckeeper-commit"]
