# Laravel Queue Worker

<!-- Merged from feature/laravel-queue-worker -->

## Idea

### Request

Provision a working per-site queue worker as standard native Laravel framework infrastructure.

The user stated the queue worker logic is currently exposed but not actually wired up.

### Problem

The Laravel framework needs a queue worker to consume jobs from the configured queue. The worker systemd unit, provisioning logic, and manifest entries provide that infrastructure by default.

A Laravel site configured with `QUEUE_CONNECTION=database` (the template default) has no worker to process queued jobs. All queued work — mail, notifications, batch operations — silently backs up in the `jobs` table forever.

### Definitions

**Queue worker:** A long-running `php artisan queue:work` process managed as a per-site systemd service. It consumes jobs from the configured Laravel queue connection and runs under the site's runtime user and group. It differs from the FPM pool (which handles HTTP requests) and from the scheduler (which runs periodic commands on a timer).

The queue worker is always provisioned for native Laravel sites. Older `install_queue_worker` values are ignored.

**Site target:** The per-site `systemd` target unit (`{project}.target`) that collects all site services via `{project}.target.requires/`. Restarting the target restarts every registered service.

### Desired outcome

For native Laravel sites:

- A systemd queue worker unit is deployed to `/etc/systemd/system/{project}-worker.service`.
- The unit runs `php artisan queue:work` with sensible production defaults.
- The service is registered in the site target and enabled to start at boot.
- The worker restarts automatically when the site target restarts during `bonesdeploy remote deploy`.
- The manifest reports the worker service and its unit file.


### Scope

This change includes:

- A new `queue-worker.service.j2` systemd template in `assets/frameworks/laravel/infra/templates/`.
- Worker deployment logic in `laravel/infra/runtime.py`.
- Manifest entries for the worker service and its unit file in `laravel/infra/manifest.py`.
- Systemd hardening and the site's existing runtime ownership and writable paths for the worker; Laravel's native FPM path does not attach an application AppArmor profile.
- Updated documentation in `templates.md` describing the default worker.
- Python tests covering worker provisioning and manifest output.

### Constraints

- Must follow the existing `systemd.register_service` + `systemd.enable_and_start` provisioning pattern.
- The worker must run under the same runtime user and group as the Laravel FPM pool.
- The worker must use the existing systemd hardening conventions and explicitly allow writes to Laravel's shared storage and deployment log directory.
- Must work with the native backend. Docker backend is excluded from scope.

### Exclusions

This change does not include:

- Laravel Horizon or any queue dashboard alternative.
- The Laravel `schedule:run` scheduler — this is a separate concept.
- Redis, Beanstalkd, or SQS queue provisioning — the worker uses whatever connection is already configured in the site's `.env`.
- Docker backend worker support.
- Per-worker concurrency configuration or the `--queue` flag — these can be configured via `custom.py` hooks.


## Plan

### Current behavior

The Laravel queue worker is standard native Laravel infrastructure. It is provisioned without a framework variable; older `install_queue_worker` values are ignored.

No code reads or acts on the value:

- `assets/frameworks/laravel/infra/runtime.py::deploy(ctx)` installs PHP, configures FPM, renders the nginx site, and calls `custom.deploy(ctx)` which is a no-op.
- `assets/frameworks/laravel/infra/manifest.py::artifacts(ctx)` lists FPM pool, socket, log, nginx, and runtime entries, plus Docker entries when applicable. No worker entries.
- `assets/frameworks/laravel/infra/manifest.py::services(ctx)` lists only `{project}-nginx.service` (and a Docker service when applicable). No worker service.
- No worker systemd template exists under `assets/frameworks/laravel/infra/templates/`.

The existing `assets/systemd/app.service.j2` template (used by `systemd.render_app_service`) is designed for socket-based application servers with `ReadOnlyPaths=paths.current`. It is not suitable for a queue worker that needs write access to storage for logging and framework cache.

The Laravel `infra/templates/app-profile.j2` file is not used by the native PHP-FPM deployment path. Unlike server-style frameworks that call `application.deploy_server`, Laravel's `runtime.py` calls `PHP.configure_fpm_pool` directly, and that path does not attach an application AppArmor profile. There is therefore no existing Laravel application profile to reuse for the worker.

During deploy, `bonesremote service restart` restarts the site target, which restarts all services registered in `{project}.target.requires/`. Services are registered via `systemd.register_service` (creates the symlink in the requires directory) and started via `systemd.enable_and_start` (enables the unit at boot and starts it now).

### Intended behavior

For native Laravel sites:

1. **Provisioning** (`runtime.py::deploy`): After the existing FPM/nginx setup, a queue worker systemd unit is rendered from a new template, registered in the site target, enabled at boot, and started.

2. **Systemd unit** (`queue-worker.service.j2`):
   - `Type=simple`, `Restart=always`, `RestartSec=2`
   - `ExecStart=/usr/bin/php{version} artisan queue:work --sleep=3 --tries=3 --max-time=3600`
   - `User={runtime_user}`, `Group={runtime_group}`
   - `WorkingDirectory=paths.current`
   - `PartOf={project}.target`, `Before={project}.target`
   - `After=network.target apparmor.service`, `Requires=apparmor.service`
   - `PrivateTmp=yes`, `ProtectHome=yes`, `ProtectSystem=strict`, `RestrictNamespaces=yes`, `LockPersonality=yes`, `RestrictRealtime=yes`, `NoNewPrivileges=yes`, `PrivateDevices=yes`, `ProtectKernelTunables=yes`, `ProtectKernelModules=yes`, `ProtectControlGroups=yes`
   - `RestrictAddressFamilies=AF_UNIX`
   - `ReadWritePaths` includes shared Laravel storage, the current release's `bootstrap/cache`, and the site deployment log directory. `ProtectSystem=strict` keeps other paths read-only.
   - `StandardOutput=journal`, `StandardError=journal`

3. **Deploy restart**: `bonesremote service restart` reads the root-owned `{project}.target.requires/` directory, starts the site target, and explicitly restarts every registered site service after cut-over. This re-evaluates `ConditionPathExists` after `current` changes from the generic placeholder to the Laravel release without consulting framework configuration or `.env`. No `php artisan queue:restart` call is needed in prepare scripts — the existing test at `frameworks.rs:195` already forbids `queue:restart` in prepare.

4. **Manifest**: `manifest.py::artifacts()` includes the worker systemd unit file and `manifest.py::services()` includes `{project}-worker.service`.


### Approach

1. **Create `queue-worker.service.j2`** in `assets/frameworks/laravel/infra/templates/`. Base it on the existing `app.service.j2` but adapt it for queue worker semantics: no socket directory, writable storage paths, and `--sleep`, `--tries`, `--max-time` arguments.

2. **Add worker deployment in `runtime.py::deploy()`**: After the nginx site rendering, render the worker template, register it, and start it. Use `systemd.register_service` and `systemd.enable_and_start` to follow existing conventions. The worker uses the site runtime user/group and explicit shared Laravel writable paths; it does not attach an AppArmor profile.

   The worker does not need an AppArmor profile name because Laravel's native FPM path does not provision one. Its filesystem access is constrained by systemd hardening and explicit writable paths.

3. **Add manifest entries in `manifest.py`**: In both `artifacts()` and `services()`, always append the worker entries. The artifact path uses `paths.systemd_service("worker")`. The service entry is `("{project}-worker.service", "framework")`.

4. **Update `templates.md`**: Document the default worker behavior.

5. **Add Python tests**: A test verifies the worker is rendered and started without a configuration flag. Manifest tests verify the entries appear.

### Responsibilities and boundaries

| Responsibility | Owner |
|---|---|
| Worker systemd unit template | `assets/frameworks/laravel/infra/templates/queue-worker.service.j2` (new) |
| Worker deployment gate and orchestration | `assets/frameworks/laravel/infra/runtime.py::deploy()` |
| Target membership registration | `systemd.register_service` (existing) |
| Service enable and start | `systemd.enable_and_start` (existing) |
| Manifest artifact and service declarations | `assets/frameworks/laravel/infra/manifest.py` |
| Worker filesystem restrictions | `assets/frameworks/laravel/infra/templates/queue-worker.service.j2` (new) |
| Documentation | `crates/bonesdeploy/assets/skill/templates.md` |

The worker deployment belongs in `runtime.py::deploy()` rather than `custom.py::deploy()` because it is framework infrastructure, not site-custom behaviour. The `custom.py` hook remains available for site-specific logic.

### Affected areas

- `crates/bonesdeploy/assets/frameworks/laravel/infra/templates/queue-worker.service.j2` — new systemd template
- `crates/bonesdeploy/assets/frameworks/laravel/infra/runtime.py` — worker deployment gate
- `crates/bonesdeploy/assets/frameworks/laravel/infra/manifest.py` — artifact and service entries
- `crates/bonesdeploy/assets/skill/templates.md` — documentation
- `crates/bonesinfra/python/tests/` — new or expanded tests for worker provisioning and manifest

### Decisions

**Worker deploys from `runtime.py::deploy()`, not `custom.py::deploy()`.** The worker is a framework concern (it needs a specific template, knows the PHP version, and uses framework-level systemd utilities). `custom.py` remains a site-specific extension seam.

**Worker does not attach an AppArmor profile.** The native Laravel PHP-FPM path does not attach an application profile, and the framework-local `app-profile.j2` is unused there. Adding a new profile is outside this queue-worker change. The unit instead uses systemd hardening and explicit writable paths for shared storage, current-release bootstrap cache, and deployment logs.

**Worker uses `Restart=always`, not `Restart=on-failure`.** The worker is designed to run continuously. `php artisan queue:work` exits with code 0 on `--max-time` expiry, which `Restart=always` handles correctly. Using `Restart=always` also matches the existing `app.service.j2` convention.

**Worker uses the project-level PHP version from `php_version` framework var.** The `ExecStart` path is `/usr/bin/php{version} artisan queue:work ...`, matching how the project is configured.

**No `queue:restart` in prepare scripts.** The existing test at `frameworks.rs:195` already asserts prepare must NOT run `queue:restart`. BonesRemote explicitly restarts an enabled Laravel worker after activating the release, so the worker sees the new `current` target and re-evaluates its `artisan` condition.

**Default worker arguments are hardcoded in the template**: `--sleep=3 --tries=3 --max-time=3600`. These are sensible production defaults. Site-specific overrides belong in `custom.py` or supervisor-level configuration, not in the framework template.

### Risks

- **Memory accumulation**: Long-running workers can accumulate memory from framework service container state. The `--max-time=3600` flag ensures workers restart hourly, releasing any accumulated memory.
- **Job loss during deploy restart**: When the site target restarts, the worker is killed. Jobs currently being processed may be lost. This is Laravel's standard behaviour with `queue:work` — jobs are marked as attempted after successful completion. Long-running jobs should use `--timeout` and be idempotent on retry. Not specific to this change.
- **Access to custom writable paths**: If a site has custom writable paths beyond Laravel's shared storage, current-release bootstrap cache, and deployment logs, the worker will not receive them automatically. The unit declares only the paths required by the template and does not broaden access to arbitrary project paths.

### Validation

- Python test: provisioning without a worker flag renders the worker unit file and registers/starts the service.
- Python manifest test: worker artifacts and service are always present for native Laravel.
- Existing `frameworks.rs:195` test continues to pass (no `queue:restart` in prepare).
- `ruff check .` and `ruff format .` pass in the Python package.
- `cargo clippy`, `cargo fmt` pass in the Rust workspace.
- `cargo test --workspace --exclude e2e` passes.
- Review the systemd template for correctness (no unbound Jinja2 variables, proper hardening directives).


## Tasks

### Implementation

- [x] Create `assets/frameworks/laravel/infra/templates/queue-worker.service.j2`: systemd unit template based on `app.service.j2` but adapted for `php artisan queue:work` with writable storage paths, `--sleep=3 --tries=3 --max-time=3600`, and no socket directory.
- [x] Add worker deployment in `assets/frameworks/laravel/infra/runtime.py::deploy()`: after nginx site rendering, render the worker template with explicit shared Laravel writable paths, register via `systemd.register_service`, and start via `systemd.enable_and_start` without an AppArmor profile.
- [x] Add worker manifest artifacts in `assets/frameworks/laravel/infra/manifest.py::artifacts()`: append the worker systemd unit file path using `paths.systemd_service("worker")`.
- [x] Add worker manifest services in `assets/frameworks/laravel/infra/manifest.py::services()`: append `("{project}-worker.service", "framework")`.
- [x] Update `crates/bonesdeploy/assets/skill/templates.md`: describe the default worker infrastructure.

### Validation

- [x] Add a Python test verifying that provisioning without a worker configuration flag renders `{project}-worker.service`, registers it, and starts the service.
- [x] Add a Python manifest test verifying worker artifact and service entries appear when enabled and are absent when disabled.
- [x] Run `ruff check .` and `ruff format .` in `crates/bonesinfra/python/` and address all warnings.
- [x] Run `uv run pytest` in `crates/bonesinfra/python/` and confirm all existing and new tests pass.
- [x] Run `cargo clippy`, `cargo fmt` from workspace root and address all warnings.
- [x] Run `cargo test --workspace --exclude e2e` and confirm all tests pass, especially the existing `frameworks.rs:195` laravel prepare test.

### Completion

- [x] Review the systemd template for unbound Jinja2 variables, correct hardening directives, and proper `ReadWritePaths` coverage.
- [x] Review the final diff to ensure the worker is always provisioned for native Laravel.
- [x] Confirm `templates.md` documentation accurately describes the option.
- [x] Extend the Laravel e2e setup to assert the per-site worker service after setup and deploy.
- [x] Restart every validated service registered in `{project}.target.requires/` after release activation so the worker's `artisan` condition is re-evaluated after the placeholder is replaced.

### Completion notes

Implemented the Laravel native queue worker as standard native Laravel infrastructure. The worker is a per-site systemd service registered in the site target, uses the selected PHP executable and explicit Laravel writable paths, and is always included in the manifest. Repository investigation clarified that the native Laravel FPM path does not attach an AppArmor profile, so the worker uses systemd hardening instead of profile reuse. BonesRemote reads the validated site target membership directory and restarts all registered services after activation because the generic placeholder release intentionally does not contain Laravel's `artisan` entrypoint.

Validation passed with `ruff check .`, `ruff format .`, `uv run pytest` (259 tests), `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --exclude e2e`, and `shfmt -w .`. E2E tests were intentionally not run.


## Clarification

### Trigger

Repository investigation after Acta planning showed that Laravel's native PHP-FPM deployment path does not render or attach an application AppArmor profile. The framework-local `app-profile.j2` is unused by `laravel/infra/runtime.py`.

### Decision

The queue worker will not attach an AppArmor profile. It will run with the existing systemd hardening conventions, the Laravel site runtime user and group, strict systemd filesystem protection, and explicit write access to shared Laravel storage, current-release bootstrap cache, and the site deployment log directory. This matches the actual Laravel runtime boundary without adding an unrelated profile subsystem.

### Supersedes

The clarification supersedes the `01-idea.md` scope item and `02-plan.md` approach and decision stating that the worker reuses an existing Laravel FPM AppArmor profile.

### Effect on the record

`01-idea.md` now defines systemd hardening and explicit writable paths as the worker's security boundary. `02-plan.md` now records that no Laravel application profile exists on this path and removes the profile reuse step. `03-tasks.md` now requires worker provisioning without an AppArmor profile and with explicit shared Laravel writable paths.


## Clarification

### Trigger

The Laravel e2e setup was requested to exercise the new queue worker without running the e2e suite locally.

### Decision

The Laravel e2e scenario will use the default worker provisioning, assert that `{site}-worker.service` is loaded but condition-skipped while the placeholder release is active, then assert it is active after deployment. The existing Laravel fixture is sufficient because it already uses `QUEUE_CONNECTION=database` and includes the database queue migration.

### Supersedes

This clarification adds e2e coverage to the validation scope without changing the worker behavior or exclusions.

### Effect on the record

`03-tasks.md` now includes the completed e2e setup and service assertion task. The e2e suite remains excluded from local execution; only its source-level setup changes are made here.
