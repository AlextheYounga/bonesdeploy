# Idea

## Request

Provision a working per-site queue worker as standard native Laravel framework infrastructure.

The user stated the queue worker logic is currently exposed but not actually wired up.

## Problem

The Laravel framework needs a queue worker to consume jobs from the configured queue. The worker systemd unit, provisioning logic, and manifest entries provide that infrastructure by default.

A Laravel site configured with `QUEUE_CONNECTION=database` (the template default) has no worker to process queued jobs. All queued work — mail, notifications, batch operations — silently backs up in the `jobs` table forever.

## Definitions

**Queue worker:** A long-running `php artisan queue:work` process managed as a per-site systemd service. It consumes jobs from the configured Laravel queue connection and runs under the site's runtime user and group. It differs from the FPM pool (which handles HTTP requests) and from the scheduler (which runs periodic commands on a timer).

The queue worker is always provisioned for native Laravel sites. Older `install_queue_worker` values are ignored.

**Site target:** The per-site `systemd` target unit (`{project}.target`) that collects all site services via `{project}.target.requires/`. Restarting the target restarts every registered service.

## Desired outcome

For native Laravel sites:

- A systemd queue worker unit is deployed to `/etc/systemd/system/{project}-worker.service`.
- The unit runs `php artisan queue:work` with sensible production defaults.
- The service is registered in the site target and enabled to start at boot.
- The worker restarts automatically when the site target restarts during `bonesdeploy remote deploy`.
- The manifest reports the worker service and its unit file.


## Scope

This change includes:

- A new `queue-worker.service.j2` systemd template in `assets/frameworks/laravel/infra/templates/`.
- Worker deployment logic in `laravel/infra/runtime.py`.
- Manifest entries for the worker service and its unit file in `laravel/infra/manifest.py`.
- Systemd hardening and the site's existing runtime ownership and writable paths for the worker; Laravel's native FPM path does not attach an application AppArmor profile.
- Updated documentation in `templates.md` describing the default worker.
- Python tests covering worker provisioning and manifest output.

## Constraints

- Must follow the existing `systemd.register_service` + `systemd.enable_and_start` provisioning pattern.
- The worker must run under the same runtime user and group as the Laravel FPM pool.
- The worker must use the existing systemd hardening conventions and explicitly allow writes to Laravel's shared storage and deployment log directory.
- Must work with the native backend. Docker backend is excluded from scope.

## Exclusions

This change does not include:

- Laravel Horizon or any queue dashboard alternative.
- The Laravel `schedule:run` scheduler — this is a separate concept.
- Redis, Beanstalkd, or SQS queue provisioning — the worker uses whatever connection is already configured in the site's `.env`.
- Docker backend worker support.
- Per-worker concurrency configuration or the `--queue` flag — these can be configured via `custom.py` hooks.
