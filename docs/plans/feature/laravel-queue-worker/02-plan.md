# Plan

## Current behavior

The `install_queue_worker` option is defined in `crates/bonesdeploy/src/frameworks/laravel.rs` as a boolean question with a `// TODO: Set up queue worker.` comment. It is accepted as a framework variable during init and persisted into `bones.toml` under `[runtime]`. In Python, the value is reachable via `ctx.runtime.data.get("install_queue_worker")`.

No code reads or acts on the value:

- `assets/frameworks/laravel/infra/runtime.py::deploy(ctx)` installs PHP, configures FPM, renders the nginx site, and calls `custom.deploy(ctx)` which is a no-op.
- `assets/frameworks/laravel/infra/manifest.py::artifacts(ctx)` lists FPM pool, socket, log, nginx, and runtime entries, plus Docker entries when applicable. No worker entries.
- `assets/frameworks/laravel/infra/manifest.py::services(ctx)` lists only `{project}-nginx.service` (and a Docker service when applicable). No worker service.
- No worker systemd template exists under `assets/frameworks/laravel/infra/templates/`.

The existing `assets/systemd/app.service.j2` template (used by `systemd.render_app_service`) is designed for socket-based application servers with `ReadOnlyPaths=paths.current`. It is not suitable for a queue worker that needs write access to storage for logging and framework cache.

The Laravel `infra/templates/app-profile.j2` file is not used by the native PHP-FPM deployment path. Unlike server-style frameworks that call `application.deploy_server`, Laravel's `runtime.py` calls `PHP.configure_fpm_pool` directly, and that path does not attach an application AppArmor profile. There is therefore no existing Laravel application profile to reuse for the worker.

During deploy, `bonesremote service restart` restarts the site target, which restarts all services registered in `{project}.target.requires/`. Services are registered via `systemd.register_service` (creates the symlink in the requires directory) and started via `systemd.enable_and_start` (enables the unit at boot and starts it now).

## Intended behavior

When `install_queue_worker` is `true` in `bones.toml` (accessible as `ctx.runtime.data.get("install_queue_worker")`):

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

4. **Manifest**: `manifest.py::artifacts()` includes the worker systemd unit file. `manifest.py::services()` includes `{project}-worker.service`. Both gated on `install_queue_worker`.

When `install_queue_worker` is `false` or absent, behavior is unchanged — no worker provisioned, no manifest entries.

## Approach

1. **Create `queue-worker.service.j2`** in `assets/frameworks/laravel/infra/templates/`. Base it on the existing `app.service.j2` but adapt it for queue worker semantics: no socket directory, writable storage paths, and `--sleep`, `--tries`, `--max-time` arguments.

2. **Add worker deployment in `runtime.py::deploy()`**: After the nginx site rendering, check `install_queue_worker`. If true, render the worker template, register it, and start it. Use `systemd.register_service` and `systemd.enable_and_start` to follow existing conventions. The worker uses the site runtime user/group and explicit shared Laravel writable paths; it does not attach an AppArmor profile.

   The worker does not need an AppArmor profile name because Laravel's native FPM path does not provision one. Its filesystem access is constrained by systemd hardening and explicit writable paths.

3. **Add manifest entries in `manifest.py`**: In both `artifacts()` and `services()`, append worker entries when `ctx.runtime.data.get("install_queue_worker")`. The artifact path uses `paths.systemd_service("worker")`. The service entry is `("{project}-worker.service", "framework")`.

4. **Update `templates.md`**: Remove any ambiguity about the `install_queue_worker` option being unimplemented. Document what the option provisions.

5. **Add Python tests**: A test that provisions with `install_queue_worker=true` verifies the systemd service file is rendered and the service is started. A test with `install_queue_worker=false` verifies no worker is created. Manifest tests verify the entries appear.

## Responsibilities and boundaries

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

## Affected areas

- `crates/bonesdeploy/assets/frameworks/laravel/infra/templates/queue-worker.service.j2` — new systemd template
- `crates/bonesdeploy/assets/frameworks/laravel/infra/runtime.py` — worker deployment gate
- `crates/bonesdeploy/assets/frameworks/laravel/infra/manifest.py` — artifact and service entries
- `crates/bonesdeploy/assets/skill/templates.md` — documentation
- `crates/bonesinfra/python/tests/` — new or expanded tests for worker provisioning and manifest

## Decisions

**Worker deploys from `runtime.py::deploy()`, not `custom.py::deploy()`.** The worker is a framework concern (it needs a specific template, knows the PHP version, and uses framework-level systemd utilities). `custom.py` remains a site-specific extension seam.

**Worker does not attach an AppArmor profile.** The native Laravel PHP-FPM path does not attach an application profile, and the framework-local `app-profile.j2` is unused there. Adding a new profile is outside this queue-worker change. The unit instead uses systemd hardening and explicit writable paths for shared storage, current-release bootstrap cache, and deployment logs.

**Worker uses `Restart=always`, not `Restart=on-failure`.** The worker is designed to run continuously. `php artisan queue:work` exits with code 0 on `--max-time` expiry, which `Restart=always` handles correctly. Using `Restart=always` also matches the existing `app.service.j2` convention.

**Worker uses the project-level PHP version from `php_version` framework var.** The `ExecStart` path is `/usr/bin/php{version} artisan queue:work ...`, matching how the project is configured.

**No `queue:restart` in prepare scripts.** The existing test at `frameworks.rs:195` already asserts prepare must NOT run `queue:restart`. BonesRemote explicitly restarts an enabled Laravel worker after activating the release, so the worker sees the new `current` target and re-evaluates its `artisan` condition.

**Default worker arguments are hardcoded in the template**: `--sleep=3 --tries=3 --max-time=3600`. These are sensible production defaults. Site-specific overrides belong in `custom.py` or supervisor-level configuration, not in the framework template.

## Risks

- **Memory accumulation**: Long-running workers can accumulate memory from framework service container state. The `--max-time=3600` flag ensures workers restart hourly, releasing any accumulated memory.
- **Job loss during deploy restart**: When the site target restarts, the worker is killed. Jobs currently being processed may be lost. This is Laravel's standard behaviour with `queue:work` — jobs are marked as attempted after successful completion. Long-running jobs should use `--timeout` and be idempotent on retry. Not specific to this change.
- **Access to custom writable paths**: If a site has custom writable paths beyond Laravel's shared storage, current-release bootstrap cache, and deployment logs, the worker will not receive them automatically. The unit declares only the paths required by the template and does not broaden access to arbitrary project paths.

## Validation

- Python test: provisioning with `install_queue_worker=true` renders the worker unit file and registers/starts the service.
- Python test: provisioning with `install_queue_worker=false` (or absent) does not create the worker.
- Python manifest test: worker artifacts and service appear when enabled, absent when disabled.
- Existing `frameworks.rs:195` test continues to pass (no `queue:restart` in prepare).
- `ruff check .` and `ruff format .` pass in the Python package.
- `cargo clippy`, `cargo fmt` pass in the Rust workspace.
- `cargo test --workspace --exclude e2e` passes.
- Review the systemd template for correctness (no unbound Jinja2 variables, proper hardening directives).
