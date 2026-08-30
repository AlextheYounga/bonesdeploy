from types import SimpleNamespace

from bonesinfra.cli.commands.site import directories


def test_setup_does_not_provision_the_shared_environment(monkeypatch):
    created = []
    script_calls = []
    monkeypatch.setattr(directories, "mkdir", lambda **kwargs: created.append(kwargs))
    monkeypatch.setattr(directories.server, "shell", lambda **_kwargs: None)
    monkeypatch.setattr(directories.server, "script_template", lambda **kwargs: script_calls.append(kwargs))
    ctx = SimpleNamespace(
        server=SimpleNamespace(host="192.0.2.1", port="22", ssh_user="root"),
        app=SimpleNamespace(
            project_name="atlas",
            deploy=SimpleNamespace(branch="main"),
            dns=SimpleNamespace(domain="", email="", ssl_enabled=False),
        ),
        runtime=SimpleNamespace(
            runtime_user="atlas", runtime_group="atlas", backend="native", web_root="public", data={}
        ),
        services=SimpleNamespace(services=()),
    )
    paths = {
        "site_root": "/root/.config/bonesremote/sites/atlas",
        "state_root": "/var/lib/bonesdeploy/state/atlas",
        "lock_path": "/var/lib/bonesdeploy/locks/.atlas.deployment.lock",
        "repo_parent": "/home/git",
        "repo": "/home/git/atlas.git",
        "project_root_parent": "/srv/sites",
        "project_root": "/srv/sites/atlas",
        "releases": "/srv/sites/atlas/releases",
        "shared": "/srv/sites/atlas/shared",
        "placeholder_web_root": "/srv/sites/atlas/releases/19700101_000000/public",
    }

    directories.setup_repo_and_project(ctx, paths)

    assert created[0] == {
        "name": "Ensure private BonesRemote state directory exists",
        "path": "/var/lib/bonesdeploy/state/atlas",
        "user": "git",
        "group": "git",
        "mode": "0700",
    }
    assert script_calls == []
