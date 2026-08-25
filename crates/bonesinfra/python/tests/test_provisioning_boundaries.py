from types import SimpleNamespace

from bonesinfra.cli.commands import server, site


def test_server_setup_runs_only_server_operations(monkeypatch):
    calls = []
    monkeypatch.setattr(server.packages, "install_system", lambda *_: calls.append("packages"))
    monkeypatch.setattr(server.disable_algif_aead, "configure", lambda: calls.append("hardening"))
    monkeypatch.setattr(server.image_store, "ensure_shared_store", lambda: calls.append("image-store"))
    monkeypatch.setattr(server.image_store, "seed_base_image", lambda: calls.append("base-image"))
    monkeypatch.setattr(server.firewall, "configure", lambda *_: calls.append("firewall"))
    monkeypatch.setattr(server.fail2ban, "configure", lambda *_: calls.append("fail2ban"))
    monkeypatch.setattr(server.unattended_upgrades, "configure", lambda: calls.append("upgrades"))
    monkeypatch.setattr(server.users, "ensure_deploy_user", lambda *_: calls.append("deploy-user"))
    monkeypatch.setattr(server, "_ensure_bonesremote_roots", lambda: calls.append("bonesremote-roots"))
    monkeypatch.setattr(server.bonesremote, "install", lambda *_: calls.append("bonesremote"))
    monkeypatch.setattr(server.sudoers, "install", lambda: calls.append("sudoers"))

    server.deploy_server_setup(SimpleNamespace(), "1.0.0")

    assert calls == [
        "packages",
        "hardening",
        "image-store",
        "base-image",
        "firewall",
        "fail2ban",
        "upgrades",
        "deploy-user",
        "bonesremote-roots",
        "bonesremote",
        "sudoers",
    ]


def test_site_setup_runs_only_site_base_operations(monkeypatch):
    calls = []
    ctx = SimpleNamespace(paths_dict={})
    monkeypatch.setattr(site.users, "ensure_users_and_groups", lambda *_: calls.append("identities"))
    monkeypatch.setattr(site.directories, "setup_repo_and_project", lambda *_: calls.append("directories"))
    monkeypatch.setattr(site.placeholder, "seed", lambda *_: calls.append("placeholder"))

    site.deploy_site_setup(ctx)

    assert calls == ["identities", "directories", "placeholder"]
