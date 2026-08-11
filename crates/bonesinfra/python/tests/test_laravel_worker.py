from pathlib import Path

from bonesinfra.config.context import DeployContext
from bonesinfra.manifest import collect_services, resolve_artifacts
from bonesinfra.project import load_manifest, load_runtime

INFRA = Path(__file__).parents[4] / "crates/bonesdeploy/assets/frameworks/laravel/infra"


def _config(tmp_path: Path, *, worker: bool | None = None) -> Path:
    value = "" if worker is None else f"install_queue_worker = {str(worker).lower()}\n"
    config = tmp_path / "bones.toml"
    config.write_text(
        f"""[app]
project_name = "atlas"
[app.server]
host = "example.test"
[runtime]
template = "laravel"
php_version = "8.5"
{value}[services]
services = []
"""
    )
    return config


def _runtime_context(config: Path):
    ctx = DeployContext.from_files(str(config))
    return ctx


def test_laravel_runtime_provisions_queue_worker_when_enabled(tmp_path, monkeypatch):
    config = _config(tmp_path, worker=True)
    (tmp_path / "infra").symlink_to(INFRA, target_is_directory=True)
    module = load_runtime(config)
    ctx = _runtime_context(config)
    calls = []

    monkeypatch.setattr(module.PHP, "install", lambda _ctx: "/usr/bin/php8.5")
    monkeypatch.setattr(module.PHP, "configure_fpm_pool", lambda _ctx, **_kwargs: "/run/php/php8.5-fpm-atlas.sock")
    monkeypatch.setattr(module.site, "render_php_fpm", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(module.custom, "deploy", lambda _ctx: None)
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


def test_laravel_runtime_skips_queue_worker_when_disabled(tmp_path, monkeypatch):
    config = _config(tmp_path, worker=False)
    (tmp_path / "infra").symlink_to(INFRA, target_is_directory=True)
    module = load_runtime(config)
    ctx = _runtime_context(config)
    renders = []

    monkeypatch.setattr(module.PHP, "install", lambda _ctx: "/usr/bin/php8.5")
    monkeypatch.setattr(module.PHP, "configure_fpm_pool", lambda _ctx, **_kwargs: "/run/php/php8.5-fpm-atlas.sock")
    monkeypatch.setattr(module.site, "render_php_fpm", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(module.custom, "deploy", lambda _ctx: None)
    monkeypatch.setattr(module, "render", lambda *args, **_kwargs: renders.append(args[0]))

    module.deploy(ctx)

    assert "Deploy Laravel queue worker service" not in renders


def test_laravel_manifest_declares_worker_only_when_enabled(tmp_path):
    config = _config(tmp_path, worker=True)
    (tmp_path / "infra").symlink_to(INFRA, target_is_directory=True)
    ctx = _runtime_context(config)
    project_manifest = load_manifest(config)

    artifacts = resolve_artifacts(ctx, project_manifest)
    services = collect_services(ctx, project_manifest)

    assert any(entry.name == "Laravel queue worker service" for entry in artifacts)
    assert any(entry.unit == "atlas-worker.service" for entry in services)


def test_laravel_manifest_omits_worker_when_disabled(tmp_path):
    config = _config(tmp_path, worker=False)
    (tmp_path / "infra").symlink_to(INFRA, target_is_directory=True)
    ctx = _runtime_context(config)
    project_manifest = load_manifest(config)

    artifacts = resolve_artifacts(ctx, project_manifest)
    services = collect_services(ctx, project_manifest)

    assert all(entry.name != "Laravel queue worker service" for entry in artifacts)
    assert all(entry.unit != "atlas-worker.service" for entry in services)
