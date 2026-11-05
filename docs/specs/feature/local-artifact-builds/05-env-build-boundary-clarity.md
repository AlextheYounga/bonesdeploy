# Build Environment Boundary Clarification

## Trigger

The Docker clarification described `.env.build` as excluded from local builds.
That was broader than intended. `.env.build` exists specifically so a project
can declare a small set of build-tool inputs such as `NODE_VERSION` without
exposing its runtime or production environment.

## Decision

Local Docker builds retain committed `.env.build` values as explicit build
inputs. The existing parser and reserved-name validation remain part of the
shared build contract. Fixed container-controlled and derived public metadata
continues to be supplied separately and cannot be overridden by `.env.build`.

Every `.env.build` value is treated as public. The file must not contain
passwords, tokens, private keys, private dependency credentials, or other
secrets because arbitrary build scripts and supply-chain dependencies can read
and exfiltrate any build input.

The build does not inherit the ambient host environment and does not receive the
root runtime `.env`, decrypted production variables, backup credentials, SSH
agent, credential stores, host home directory, or Docker socket.

## Supersedes

This clarification supersedes only the statements in
`04-local-only-docker-production-support-clarity.md` that excluded `.env.build`
and user-defined build variables entirely. It does not change Docker as the sole
local engine, removal of native remote builds, the breaking compatibility
boundary, or Debian 12+/Ubuntu 24.04+ production support.

## Required Authoritative Updates

- `01-idea.md` now defines `.env.build` as explicit, committed, non-secret build
  configuration and distinguishes it from excluded runtime and host variables.
- `02-plan.md` now retains `.env.build` parsing, projection, reserved-name
  validation, and tests while continuing to exclude all ambient and sensitive
  environments.
- `03-tasks.md` now requires preservation and validation of the `.env.build`
  boundary instead of its removal.
