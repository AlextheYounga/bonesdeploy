from types import SimpleNamespace

from bonesinfra.cli.commands.site import directories


def test_setup_does_not_provision_the_shared_environment(monkeypatch):
    created = []
    monkeypatch.setattr(directories, "mkdir", lambda **kwargs: created.append(kwargs))
    ctx = SimpleNamespace(runtime=SimpleNamespace(runtime_user="atlas", runtime_group="atlas"))
    paths = {
        "site_root": "/root/.config/bonesremote/sites/atlas",
        "project_root_parent": "/srv/sites",
        "project_root": "/srv/sites/atlas",
        "releases": "/srv/sites/atlas/releases",
        "shared": "/srv/sites/atlas/shared",
        "placeholder_web_root": "/srv/sites/atlas/releases/19700101_000000/public",
    }

    directories.setup_project(ctx, paths)

    assert created[0] == {
        "name": "Ensure control-plane site state directory exists",
        "path": "/root/.config/bonesremote/sites/atlas",
        "user": "root",
        "group": "root",
        "mode": "0700",
    }
