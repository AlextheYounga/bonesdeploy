# Clarification

## Trigger

The Laravel e2e setup was requested to exercise the new queue worker without running the e2e suite locally.

## Decision

The Laravel e2e scenario will provision with `install_queue_worker=true` and assert that `{site}-worker.service` is active after runtime setup and after deployment. The existing Laravel fixture is sufficient because it already uses `QUEUE_CONNECTION=database` and includes the database queue migration.

## Supersedes

This clarification adds e2e coverage to the validation scope without changing the worker behavior, opt-in configuration, or exclusions.

## Effect on the record

`03-tasks.md` now includes the completed e2e setup and service assertion task. The e2e suite remains excluded from local execution; only its source-level setup changes are made here.
