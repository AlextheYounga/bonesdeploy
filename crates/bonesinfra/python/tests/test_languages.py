from types import SimpleNamespace

import pytest

from bonesinfra.services.languages import NODE, PYTHON, RUBY


def _context(**runtime_data):
    return SimpleNamespace(runtime=SimpleNamespace(data=runtime_data))


def test_language_runtime_stores_selected_version_and_executable(monkeypatch):
    monkeypatch.setattr("bonesinfra.services.languages.node.server.script", lambda **_kwargs: None)

    executable = NODE.install(_context(node_version="24.18.0"))

    assert NODE.version == "24.18.0"
    assert NODE.executable == executable
    assert executable.endswith("/v24.18.0/bin/node")


@pytest.mark.parametrize(
    ("runtime", "key", "value"),
    [(PYTHON, "python_version", "3"), (RUBY, "ruby_version", "3.x")],
)
def test_language_runtime_rejects_non_major_minor_versions(runtime, key, value):
    with pytest.raises(ValueError, match=key):
        runtime.install(_context(**{key: value}))
