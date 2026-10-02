from types import SimpleNamespace

import pytest

from bonesinfra.frameworks.rails.runtime import apparmor_exec_paths, bundler_binary, bundler_command
from bonesinfra.services.languages import NODE, PYTHON, RUBY
from bonesinfra.services.languages.php import PHPRuntime
from bonesinfra.services.languages.python import PYTHON_PACKAGES, PythonRuntime
from bonesinfra.services.languages.ruby import RUBY_PACKAGES, RubyRuntime


def _context(**runtime_data):
    return SimpleNamespace(runtime=SimpleNamespace(data=runtime_data))


def test_language_runtime_stores_selected_version_and_executable(monkeypatch):
    monkeypatch.setattr("bonesinfra.services.languages.node.server.script", lambda **_kwargs: None)

    executable = NODE.install(_context(node_version="24.19.0"))

    assert NODE.version == "24.19.0"
    assert NODE.executable == executable
    assert executable.endswith("/v24.19.0/bin/node")


@pytest.mark.parametrize(
    ("runtime", "key", "value"),
    [(PYTHON, "python_version", "3"), (RUBY, "ruby_version", "3.x")],
)
def test_language_runtime_rejects_invalid_versions(runtime, key, value):
    with pytest.raises(ValueError, match=key):
        runtime.install(_context(**{key: value}))


def test_python_runtime_installs_distribution_packages(monkeypatch):
    calls = []
    runtime = PythonRuntime()

    monkeypatch.setattr("bonesinfra.services.languages.python.apt.packages", lambda **kwargs: calls.append(kwargs))

    executable = runtime.install(_context(python_version="3.14"))

    assert runtime.version == "3.14"
    assert executable == "/usr/bin/python3"
    assert len(calls) == 1
    assert calls[0]["packages"] == PYTHON_PACKAGES
    assert calls[0]["present"] is True
    assert calls[0]["update"] is True
    assert calls[0]["_sudo"] is True


@pytest.mark.parametrize(
    ("selected", "expected"),
    [("3.4.8", "3.4.8"), ("3.4", "3.4.8")],
)
def test_ruby_runtime_installs_distribution_packages(monkeypatch, selected, expected):
    calls = []
    monkeypatch.setattr("bonesinfra.services.languages.ruby.apt.packages", lambda **kwargs: calls.append(kwargs))

    runtime = RubyRuntime()
    executable = runtime.install(_context(ruby_version=selected))

    assert runtime.version == expected
    assert executable == "/usr/bin/ruby"
    assert len(calls) == 1
    assert calls[0]["packages"] == RUBY_PACKAGES
    assert calls[0]["present"] is True
    assert calls[0]["update"] is True
    assert calls[0]["_sudo"] is True


def test_ruby_runtime_rejects_unsupported_patch_release():
    with pytest.raises(ValueError, match="ruby_version"):
        RubyRuntime().install(_context(ruby_version="3.4.9"))


def test_rails_bundler_binary_is_next_to_distribution_ruby():
    assert bundler_binary("/usr/bin/ruby") == "/usr/bin/bundle"


def test_rails_bundler_commands_use_the_project_local_bundle():
    assert (
        bundler_command("/usr/bin/bundle", "exec puma --help")
        == "BUNDLE_PATH=vendor/bundle /usr/bin/bundle exec puma --help"
    )


def test_rails_apparmor_allows_every_puma_command_executable():
    assert apparmor_exec_paths("/usr/bin/ruby") == ["/usr/bin/env", "/usr/bin/ruby", "/usr/bin/bundle"]


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
