# Idea

## Request

Wire up the existing `install_queue_worker` option for Laravel framework templates so that enabling it provisions a working per-site queue worker.

The user stated the queue worker logic is currently exposed but not actually wired up.

## Problem

The `install_queue_worker` framework variable is defined as a boolean question during `bonesdeploy init`, persisted into `bones.toml`, and documented in `templates.md`. But no code reads the value or acts on it. The `custom.py` deploy hook is a no-op. No systemd unit template exists for a queue worker. The manifest does not declare any worker artifacts or services.

A Laravel site configured with `QUEUE_CONNECTION=database` (the template default) has no worker to process queued jobs. All queued work — mail, notifications, batch operations — silently backs up in the `jobs` table forever.

## Definitions

**Queue worker:** A long-running `php artisan queue:work` process managed as a per-site systemd service. It consumes jobs from the configured Laravel queue connection and runs under the site's runtime user and group. It differs from the FPM pool (which handles HTTP requests) and from the scheduler (which runs periodic commands on a timer).

**install_queue_worker:** The existing boolean framework variable stored in `[runtime]` that controls whether the queue worker infrastructure is provisioned. Default is `false`.

**Site target:** The per-site `systemd` target unit (`{project}.target`) that collects all site services via `{project}.target.requires/`. Restarting the target restarts every registered service.

## Desired outcome

When `install_queue_worker` is `true` in `bones.toml`:

- A systemd queue worker unit is deployed to `/etc/systemd/system/{project}-worker.service`.
- The unit runs `php artisan queue:work` with sensible production defaults.
- The service is registered in the site target and enabled to start at boot.
- The worker restarts automatically when the site target restarts during `bonesdeploy remote deploy`.
- The manifest reports the worker service and its unit file.

When `install_queue_worker` is `false` or absent, no worker is provisioned and no worker artifacts appear in the manifest.

## Scope

This change includes:

- A new `queue-worker.service.j2` systemd template in `assets/frameworks/laravel/infra/templates/`.
- Worker deployment logic in `laravel/infra/runtime.py`, gated on `install_queue_worker`.
- Manifest entries for the worker service and its unit file in `laravel/infra/manifest.py`.
- Systemd hardening and the site's existing runtime ownership and writable paths for the worker; Laravel's native FPM path does not attach an application AppArmor profile.
- Updated documentation in `templates.md` confirming the option is functional.
- Python tests covering worker provisioning, absence when disabled, and manifest output.

## Constraints

- Must use the existing `install_queue_worker` boolean framework variable. Do not introduce a new option key.
- Must remain opt-in. The default (`false`) must not change.
- Must follow the existing `systemd.register_service` + `systemd.enable_and_start` provisioning pattern.
- The worker must run under the same runtime user and group as the Laravel FPM pool.
- The worker must use the existing systemd hardening conventions and explicitly allow writes to Laravel's shared storage and deployment log directory.
- Must work with the native backend. Docker backend is excluded from scope.

## Exclusions

This change does not include:

- Laravel Horizon or any queue dashboard alternative.
- The Laravel `schedule:run` scheduler — this is a separate concept.
- Redis, Beanstalkd, or SQS queue provisioning — the worker uses whatever connection is already configured in the site's `.env`.
- Changing the default value of `install_queue_worker` from `false` to `true`.
- Docker backend worker support.
- Per-worker concurrency configuration or the `--queue` flag — these can be configured via `custom.py` hooks.
