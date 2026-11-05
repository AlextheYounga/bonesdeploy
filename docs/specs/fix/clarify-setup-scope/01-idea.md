# Idea

## Request

Improve the CLI help and agent documentation so agents do not confuse the
host-wide server setup with per-project site setup.

## Problem

The CLI describes `bonesdeploy setup`, `bonesdeploy server setup`, and
`bonesdeploy site setup` without consistently stating their operational scope.
After initialization, the CLI specifically directs every project to run server
setup. On a shared host this encourages redundant concurrent host provisioning,
which can contend on host-wide resources before any sites are provisioned.

## Definitions

**Server setup:** Host-wide provisioning performed once for a shared host.

**Site setup:** Project-scoped provisioning performed once for each project on
that host.

**Combined setup:** `bonesdeploy setup`, a convenience command for a project's
first setup on a fresh host that runs server setup and then site setup.

## Desired outcome

CLI users and agents can determine from command help and bundled guidance that
server setup runs once per host, site setup runs once per project, and additional
projects on a prepared host should run site setup directly.

## Scope

Update setup command help, initialization follow-up output, embedded agent
documentation, and the README. Add command-level regression tests for the
rendered guidance.

## Constraints

Keep existing command names, flags, orchestration, and provisioning behavior.
Use the existing Clap help and integration-test patterns. Do not run e2e tests.

## Exclusions

Do not alter setup idempotency, add concurrency controls, change tunnel
provisioning, or redesign setup orchestration.
