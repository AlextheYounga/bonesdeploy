# Clarification

## Trigger

The Laravel e2e setup was requested to exercise the new queue worker without running the e2e suite locally.

## Decision

The Laravel e2e scenario will use the default worker provisioning, assert that `{site}-worker.service` is loaded but condition-skipped while the placeholder release is active, then assert it is active after deployment. The existing Laravel fixture is sufficient because it already uses `QUEUE_CONNECTION=database` and includes the database queue migration.

## Supersedes

This clarification adds e2e coverage to the validation scope without changing the worker behavior or exclusions.

## Effect on the record

`03-tasks.md` now includes the completed e2e setup and service assertion task. The e2e suite remains excluded from local execution; only its source-level setup changes are made here.
