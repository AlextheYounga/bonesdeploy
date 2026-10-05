from types import SimpleNamespace

import pytest

from bonesinfra.frameworks.django import runtime as django


def test_placeholder_uses_the_configured_wsgi_module_path(monkeypatch):
    captured = {}
    operations = []
    ctx = SimpleNamespace(
        runtime=SimpleNamespace(data={"wsgi_module": "djangotest.wsgi:application"}, runtime_group="site"),
        paths_dict={},
    )
    paths = {
        "current": "/srv/sites/example/current",
        "placeholder_release": "/srv/sites/example/releases/19700101_000000",
        "runtime_socket_dir": "/run/example",
    }

    monkeypatch.setattr(django.runtime, "orchestrate", lambda current_ctx, provision: provision(current_ctx))
    monkeypatch.setattr(django.shared, "ensure_directories", lambda *_args: None)
    monkeypatch.setattr(django.application, "deploy_server", lambda _ctx, **kwargs: captured.update(kwargs))
    monkeypatch.setattr(django.server, "shell", lambda **kwargs: operations.append(kwargs))
    monkeypatch.setattr(django, "mkdir", lambda **kwargs: operations.append(kwargs))
    monkeypatch.setattr(django, "render", lambda *args, **kwargs: operations.append((args, kwargs)))
    monkeypatch.setattr(django, "template_data", lambda *_args, **_kwargs: {})

    django.deploy(ctx)
    captured["seed_placeholder"](ctx, paths, "/srv/sites/example/.bonesdeploy/runtimes/python/bin/python")

    assert operations[1]["path"] == "/srv/sites/example/releases/19700101_000000/djangotest"
    assert operations[2][0][2] == "/srv/sites/example/releases/19700101_000000/djangotest/wsgi.py"
    assert "/srv/sites/example/.bonesdeploy/runtimes/python/bin/python" in operations[0]["commands"][0]
    assert "pip install" not in operations[0]["commands"][0]
    assert "djangotest.wsgi:application" in captured["command"](ctx, paths, None)


@pytest.mark.parametrize(
    "module",
    [
        "../../tmp/wsgi:application",
        "config.wsgi:application; touch /tmp/pwn",
        "config.wsgi",
    ],
)
def test_django_rejects_unsafe_wsgi_modules(module):
    ctx = SimpleNamespace(runtime=SimpleNamespace(data={"wsgi_module": module}))

    with pytest.raises(ValueError, match=r"package\.module:callable"):
        django._wsgi_module(ctx)


def test_gunicorn_uses_its_runtime_directory_for_worker_temp(monkeypatch):
    captured = {}
    ctx = SimpleNamespace(runtime=SimpleNamespace(data={}), paths_dict={})
    paths = {
        "current": "/srv/sites/example/current",
        "runtime_socket_dir": "/run/example",
    }

    monkeypatch.setattr(django.runtime, "orchestrate", lambda current_ctx, provision: provision(current_ctx))
    monkeypatch.setattr(django.shared, "ensure_directories", lambda *_args: None)
    monkeypatch.setattr(django.application, "deploy_server", lambda _ctx, **kwargs: captured.update(kwargs))

    django.deploy(ctx)

    command = captured["command"](ctx, paths, "/srv/sites/example/.bonesdeploy/runtimes/python/bin/python")
    assert "--worker-tmp-dir /run/example/gunicorn" in command


def test_gunicorn_uses_the_site_python_runtime_and_release_packages(monkeypatch):
    captured = {}
    ctx = SimpleNamespace(runtime=SimpleNamespace(data={}), paths_dict={})
    paths = {
        "current": "/srv/sites/example/current",
        "runtime_socket_dir": "/run/example",
        "shared": "/srv/sites/example/shared",
    }

    monkeypatch.setattr(django.runtime, "orchestrate", lambda current_ctx, provision: provision(current_ctx))
    monkeypatch.setattr(django.shared, "ensure_directories", lambda *_args: None)
    monkeypatch.setattr(django.application, "deploy_server", lambda _ctx, **kwargs: captured.update(kwargs))

    django.deploy(ctx)

    assert captured["command"](ctx, paths, "/srv/sites/example/.bonesdeploy/runtimes/python/bin/python").startswith(
        "PYTHONPATH=/srv/sites/example/current/.python-packages "
        "/srv/sites/example/.bonesdeploy/runtimes/python/bin/python -m gunicorn "
    )
    assert captured["exec_paths"](ctx, paths, "/srv/sites/example/.bonesdeploy/runtimes/python/bin/python") == [
        "/srv/sites/example/.bonesdeploy/runtimes/python/bin/python"
    ]
    assert captured["writable_paths"](ctx, paths) == ["/srv/sites/example/shared/media"]
