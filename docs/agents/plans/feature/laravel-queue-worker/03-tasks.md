# Tasks

## Implementation

- [x] Create `assets/frameworks/laravel/infra/templates/queue-worker.service.j2`: systemd unit template based on `app.service.j2` but adapted for `php artisan queue:work` with writable storage paths, `--sleep=3 --tries=3 --max-time=3600`, and no socket directory.
- [x] Add worker deployment gate in `assets/frameworks/laravel/infra/runtime.py::deploy()`: after nginx site rendering, check `ctx.runtime.data.get("install_queue_worker")`, render the worker template with explicit shared Laravel writable paths, register via `systemd.register_service`, and start via `systemd.enable_and_start` without an AppArmor profile.
- [x] Add worker manifest artifacts in `assets/frameworks/laravel/infra/manifest.py::artifacts()`: when `install_queue_worker` is true, append the worker systemd unit file path using `paths.systemd_service("worker")`.
- [x] Add worker manifest services in `assets/frameworks/laravel/infra/manifest.py::services()`: when `install_queue_worker` is true, append `("{project}-worker.service", "framework")`.
- [x] Update `crates/bonesdeploy/assets/skill/templates.md`: confirm `install_queue_worker` is functional and describe what it provisions.

## Validation

- [x] Add a Python test verifying that provisioning with `install_queue_worker=true` in `ctx.runtime.data` renders `{project}-worker.service`, registers it, and starts the service.
- [x] Add a Python test verifying that provisioning with `install_queue_worker=false` (or absent) does not create any worker systemd unit or service registration.
- [x] Add a Python manifest test verifying worker artifact and service entries appear when enabled and are absent when disabled.
- [x] Run `ruff check .` and `ruff format .` in `crates/bonesinfra/python/` and address all warnings.
- [x] Run `uv run pytest` in `crates/bonesinfra/python/` and confirm all existing and new tests pass.
- [x] Run `cargo clippy`, `cargo fmt` from workspace root and address all warnings.
- [x] Run `cargo test --workspace --exclude e2e` and confirm all tests pass, especially the existing `frameworks.rs:195` laravel prepare test.

## Completion

- [x] Review the systemd template for unbound Jinja2 variables, correct hardening directives, and proper `ReadWritePaths` coverage.
- [x] Review the final diff to ensure no accidental behaviour change when `install_queue_worker` is absent or false.
- [x] Confirm `templates.md` documentation accurately describes the option.
- [x] Extend the Laravel e2e setup to provision `install_queue_worker=true` and assert the per-site worker service after setup and deploy.

## Completion notes

Implemented the Laravel native queue worker behind the existing opt-in `install_queue_worker` runtime variable. The worker is a per-site systemd service registered in the site target, uses the selected PHP executable and explicit Laravel writable paths, and is included in the manifest only when enabled. Repository investigation clarified that the native Laravel FPM path does not attach an AppArmor profile, so the worker uses systemd hardening instead of profile reuse.

Validation passed with `ruff check .`, `ruff format .`, `uv run pytest` (259 tests), `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --exclude e2e`, and `shfmt -w .`. E2E tests were intentionally not run.
