# Clarification

## Trigger

The service environment contract was clarified: Redis and Valkey use the
normal default port, service values are injected only during first
initialization, and Laravel must receive framework-native database settings
before its encrypted environment is created.

## Decision

Use port `6379` for Redis and Valkey by default. Provisioning fails when the
requested port is occupied and never selects a replacement automatically.

On first initialization, BonesDeploy renders the framework environment,
injects values for services selected in the root `.env`, validates the complete
environment, and encrypts it. For Laravel with PostgreSQL, the injected values
include `DB_CONNECTION=pgsql`, `DB_HOST=127.0.0.1`, `DB_PORT=5432`,
`DB_DATABASE`, `DB_USERNAME`, and generated `DB_PASSWORD`. `APP_KEY` remains
blank; the user runs `app:key-generate` locally and enters the result through
`bonesdeploy secrets`.

If `infra/secrets/.env.gpg` already exists, initialization returns without
reading or changing it. Later service additions are handled manually through
`bonesdeploy secrets edit`. Existing remote service values are not migrated.

Store the sanitized remote control-plane copy as validated JSON at
`/srv/conf/<site>/bones.json`, atomically replaced by BonesDeploy and read by
remote-only BonesRemote actions.

## Supersedes

This replaces the earlier plan for encrypted-environment reconciliation, the
unsettled Redis/Valkey port allocation, and the `bones.toml` filename and
location. It adds the framework-native Laravel PostgreSQL injection contract.

## Effect on the record

`01-idea.md` now defines first-init-only injection, port `6379`, no service
migration, Laravel-native PostgreSQL values, and remote `bones.json`.

`02-plan.md` now specifies the single first-init flow, fail-on-port-conflict
behavior, and `/srv/conf/<site>/bones.json` synchronization.

`03-tasks.md` now tracks first-init injection and `bones.json` validation
instead of reconciliation and `bones.toml` work.
