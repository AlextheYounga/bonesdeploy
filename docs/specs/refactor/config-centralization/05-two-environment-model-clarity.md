# Clarification

## Trigger

The root `.env` must also serve as the application's local environment. It
must not be replaced when it already exists. The encrypted runtime environment
is conceptually the production environment, and local-only setup values must
not be included in it.

## Decision

Model two explicit concepts in Rust and related code:

- `LocalEnvironment` is root `.env`, conceptually `.env.local`. It contains
  application-local values plus a comment-delimited, BonesDeploy-managed
  `BONES_*` block.
- `ProductionEnvironment` is the plaintext represented by
  `infra/secrets/.env.gpg`, conceptually `.env.production`, and published as
  remote `shared/.env`. It contains application values only.

Initialization preserves all application-owned local text and values. It adds
missing framework keys and blank local service keys, then atomically replaces
the managed `BONES_*` block. The production environment is derived from the
local application key set after removing `BONES_*` keys. Generated service
values override framework defaults in production; existing local values are
never overwritten.

The `BONES_*` namespace is the machine-enforced production exclusion boundary.
The comment delimiters make managed ownership clear to users. Rust remains the
sole root `.env` parser, and BonesInfra receives stdin typed requests rather
than a file path.

## Supersedes

This replaces the earlier description of root `.env` as a control-plane-only
file. It refines first-init injection so only production service values override
framework defaults; local application values remain user-owned.

## Effect on the record

`01-idea.md` now defines local and production environments, the managed
`BONES_*` block, and the no-overwrite local merge rule.

`02-plan.md` now requires Rust-owned environment separation, atomic managed
block replacement, stdin requests for all BonesInfra commands, and production
filtering of `BONES_*` values.

`03-tasks.md` now tracks environment modeling, managed-block merge behavior,
legacy file-transport removal, and local/production boundary regression tests.
