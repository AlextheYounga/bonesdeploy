from types import SimpleNamespace

import pytest

from bonesinfra.frameworks.rails.runtime import apparmor_exec_paths, bundled_bundler, bundled_bundler_command
from bonesinfra.services.languages import NODE, PYTHON, RUBY
from bonesinfra.services.languages.php import PHPRuntime
from bonesinfra.services.languages.python import PythonRuntime
from bonesinfra.services.languages.ruby import RubyRuntime


def _context(**runtime_data):
    return SimpleNamespace(
        paths=SimpleNamespace(
            node_runtime="/srv/sites/example/.bonesdeploy/runtimes/node",
            python_runtime="/srv/sites/example/.bonesdeploy/runtimes/python",
            ruby_runtime="/srv/sites/example/.bonesdeploy/runtimes/ruby",
        ),
        runtime=SimpleNamespace(data=runtime_data, runtime_user="example"),
    )


def test_language_runtime_stores_selected_version_and_executable(monkeypatch):
    monkeypatch.setattr("bonesinfra.services.languages.mise.server.script", lambda **_kwargs: None)
    monkeypatch.setattr("bonesinfra.services.languages.mise.server.shell", lambda **_kwargs: None)

    executable = NODE.install(_context(node_version="24.19.0"))

    assert NODE.version == "24.19.0"
    assert NODE.executable == executable
    assert executable == "/srv/sites/example/.bonesdeploy/runtimes/node/bin/node"


@pytest.mark.parametrize(
    ("runtime", "key", "value"),
    [(PYTHON, "python_version", "3"), (RUBY, "ruby_version", "3.x")],
)
def test_language_runtime_rejects_invalid_versions(runtime, key, value):
    with pytest.raises(ValueError, match=key):
        runtime.install(_context(**{key: value}))


@pytest.mark.parametrize(
    ("runtime", "key", "selected", "expected"),
    [
        (NODE, "node_version", "24.19.0", "node"),
        (PythonRuntime(), "python_version", "3.14.0", "python"),
        (RubyRuntime(), "ruby_version", "3.4.9", "ruby"),
    ],
)
def test_managed_runtime_installs_an_exact_version_and_returns_its_site_link(
    monkeypatch, runtime, key, selected, expected
):
    calls = []
    monkeypatch.setattr(
        "bonesinfra.services.languages.mise.server.script", lambda **kwargs: calls.append(("script", kwargs))
    )
    monkeypatch.setattr(
        "bonesinfra.services.languages.mise.server.shell", lambda **kwargs: calls.append(("shell", kwargs))
    )

    executable = runtime.install(_context(**{key: selected}))

    assert runtime.version == selected
    assert executable == f"/srv/sites/example/.bonesdeploy/runtimes/{expected}/bin/{expected}"
    assert calls[0][1]["args"] == (expected, selected, f"/srv/sites/example/.bonesdeploy/runtimes/{expected}")
    assert calls[0][1]["_sudo"] is True
    assert calls[0][1]["_env"]["MISE_ALL_COMPILE"] == "false"
    assert calls[1][1]["_sudo_user"] == "example"
    assert selected in calls[1][1]["commands"][0]

    access = runtime.apparmor_access()
    assert access.executable == f"/var/lib/bonesdeploy/mise/installs/{expected}/{selected}/bin/{expected}"
    assert access.root == f"/var/lib/bonesdeploy/mise/installs/{expected}/{selected}"


def test_rails_bundler_is_packaged_with_the_release():
    assert (
        bundled_bundler({"current": "/srv/sites/atlas/current"})
        == "/srv/sites/atlas/current/vendor/bundle/bundler/bin/bundle"
    )


def test_rails_bundler_commands_use_managed_ruby_and_the_packaged_bundler():
    assert (
        bundled_bundler_command(
            "/srv/sites/atlas/.bonesdeploy/runtimes/ruby/bin/ruby",
            "/srv/sites/atlas/current/vendor/bundle/bundler/bin/bundle",
            "exec puma --help",
        )
        == "BUNDLE_DISABLE_VERSION_CHECK=true BUNDLE_PATH=vendor/bundle "
        "/srv/sites/atlas/.bonesdeploy/runtimes/ruby/bin/ruby "
        "/srv/sites/atlas/current/vendor/bundle/bundler/bin/bundle exec puma --help"
    )


def test_rails_apparmor_allows_every_puma_command_executable():
    paths = {"releases": "/srv/sites/atlas/releases"}

    assert apparmor_exec_paths(paths, "/srv/sites/atlas/.bonesdeploy/runtimes/ruby/bin/ruby") == [
        "/usr/bin/env",
        "/srv/sites/atlas/.bonesdeploy/runtimes/ruby/bin/ruby",
        "/srv/sites/atlas/releases/*/vendor/bundle/bundler/bin/bundle",
        "/srv/sites/atlas/releases/*/vendor/bundle/ruby/*/bin/puma",
    ]


def test_php_runtime_configures_the_project_fpm_pool(monkeypatch):
    runtime = PHPRuntime()
    runtime.version = "8.5"
    calls = {}
    ctx = SimpleNamespace(
        app=SimpleNamespace(project_name="atlas"),
        runtime=SimpleNamespace(runtime_user="atlas", runtime_group="atlas"),
    )

    monkeypatch.setattr("bonesinfra.services.languages.php.logs.ensure", lambda ctx: calls.setdefault("logs", ctx))
    monkeypatch.setattr(
        "bonesinfra.services.languages.php.server.script_template", lambda **kwargs: calls.setdefault("cleanup", kwargs)
    )
    monkeypatch.setattr(
        "bonesinfra.services.languages.php.files.template", lambda **kwargs: calls.setdefault("pool", kwargs)
    )
    monkeypatch.setattr(
        "bonesinfra.services.languages.php.server.shell", lambda **kwargs: calls.setdefault("validation", kwargs)
    )
    monkeypatch.setattr(
        "bonesinfra.services.languages.php.systemd.service", lambda **kwargs: calls.setdefault("service", kwargs)
    )
    monkeypatch.setattr("bonesinfra.services.languages.php.template_data", lambda _ctx, **_kwargs: {})

    socket_path = runtime.configure_fpm_pool(ctx, paths={"current": "/srv/sites/atlas/current"})

    assert socket_path == "/run/php/php8.5-fpm-atlas.sock"
    assert calls["cleanup"]["current_pool"] == "/etc/php/8.5/fpm/pool.d/atlas.conf"
    assert calls["pool"]["dest"] == "/etc/php/8.5/fpm/pool.d/atlas.conf"
    assert calls["pool"]["php_fpm_socket_path"] == socket_path
    assert calls["validation"]["commands"] == ["php-fpm8.5 --test"]
    assert calls["service"]["service"] == "php8.5-fpm"
