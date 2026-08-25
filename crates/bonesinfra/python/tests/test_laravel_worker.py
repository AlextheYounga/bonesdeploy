from pathlib import Path

from bonesinfra.config.context import DeployContext
from bonesinfra.manifest import collect_services, resolve_artifacts
from bonesinfra.project import load_manifest, load_runtime

INFRA = Path(__file__).parents[1] / "src/bonesinfra/frameworks/laravel"


def _config(tmp_path: Path) -> Path:
    config = tmp_path / ".env"
    config.write_text(
        """PROJECT_NAME=atlas
HOST=example.test
TEMPLATE=laravel
php_version=8.5
SERVICES=
"""
    )
    return config


def _runtime_context(config: Path):
    ctx = DeployContext.from_files(str(config))
    return ctx


def _link_core(tmp_path: Path):
    core = tmp_path / "infra/.framework/src/bonesinfra"
    core.mkdir(parents=True)
    (core / "__main__.py").write_text("")


def test_laravel_runtime_provisions_queue_worker_when_enabled(tmp_path, monkeypatch):
    config = _config(tmp_path)
    _link_core(tmp_path)
    module = load_runtime(config)
    ctx = _runtime_context(config)
    calls = []

    monkeypatch.setattr(module.PHP, "install", lambda _ctx: "/usr/bin/php8.5")
    monkeypatch.setattr(module.PHP, "configure_fpm_pool", lambda _ctx, **_kwargs: "/run/php/php8.5-fpm-atlas.sock")
    monkeypatch.setattr(module.site, "render_php_fpm", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(module.shared, "ensure_directories", lambda *_args: None)
    monkeypatch.setattr(module.runtime, "setup", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(module.runtime, "reconcile_ingress", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(module.runtime, "start_services", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(module, "render", lambda *args, **kwargs: calls.append((args, kwargs)))
    monkeypatch.setattr(
        module.systemd, "register_service", lambda *args, **kwargs: calls.append(("register", args, kwargs))
    )
    monkeypatch.setattr(
        module.systemd, "enable_and_start", lambda *args, **kwargs: calls.append(("start", args, kwargs))
    )

    module.deploy(ctx)

    worker_render = next(
        (args, kwargs) for args, kwargs in calls if args and args[0] == "Deploy Laravel queue worker service"
    )
    assert worker_render[0][2] == "/etc/systemd/system/atlas-worker.service"
    assert worker_render[1]["php_executable"] == "/usr/bin/php8.5"
    assert any(call[0] == "register" and call[2]["name"] == "worker" for call in calls)
    assert any(call[0] == "start" and call[1][1] == "worker" for call in calls)

    template = (INFRA / "templates/queue-worker.service.j2").read_text()
    assert "artisan queue:work --sleep=3 --tries=3 --max-time=3600" in template
    assert "{{ paths.shared }}/storage" in template
    assert "{{ paths.current }}/bootstrap/cache" in template
    assert "ProtectSystem=strict" in template
    assert "ConditionPathExists={{ paths.current }}/artisan" in template


def test_laravel_manifest_declares_worker_without_configuration_flag(tmp_path):
    config = _config(tmp_path)
    _link_core(tmp_path)
    ctx = _runtime_context(config)
    project_manifest = load_manifest(config)

    artifacts = resolve_artifacts(ctx, project_manifest)
    services = collect_services(ctx, project_manifest)

    assert any(entry.name == "Laravel queue worker service" for entry in artifacts)
    assert any(entry.unit == "atlas-worker.service" for entry in services)
