from types import SimpleNamespace

from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.frameworks.angular import runtime as angular


def test_angular_uses_standard_bonesdeploy_placeholder(monkeypatch):
    calls = []
    ctx = SimpleNamespace(paths_dict={})

    monkeypatch.setattr(angular.runtime, "orchestrate", lambda current_ctx, provision: provision(current_ctx))
    monkeypatch.setattr(angular.shared, "ensure_directories", lambda *_args: None)
    monkeypatch.setattr(angular, "deploy_static", lambda *_args, **kwargs: calls.append(kwargs))

    angular.deploy(ctx)

    assert calls[0]["placeholder_template"] == ASSETS_DIR / "nginx/index.html.j2"
