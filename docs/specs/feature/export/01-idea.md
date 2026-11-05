# Idea

## Request

Add a local command that exports a site's remote `shared/` directory as a ZIP.

## Problem

Persistent runtime data is stored remotely, but retrieving it currently requires
manual SSH archive creation and copying.

## Definitions

**Shared export:** A live ZIP of one site's remote `shared/` directory,
including hidden files. Nested symbolic links are stored rather than followed.

## Desired outcome

`bonesdeploy site export [--output <path>]` streams the remote `shared/`
directory over the existing root SSH connection into a local `0600` ZIP file.
Without an explicit filename, it uses `<site>-shared-<YYYYMMDD_HHMMSS>.zip` in
UTC. Existing files are not overwritten.

## Scope

- Local CLI arguments, destination handling, and binary SSH streaming.
- A direct, root-run remote `zip` command over the existing privileged SSH
  connection.
- Focused tests and operator documentation.

## Constraints

- Export is local administrative functionality; it uses the existing configured
  root SSH connection, not the `git` deployment identity.
- The archive is a live best-effort view and must not stop the site.
- `shared/.env` is included and the resulting local archive is sensitive.
- Do not run E2E tests.

## Exclusions

- No `bonesremote` command, sudoers rule, BonesInfra change, or wheel update.
- No import, restore, Borg operation, remote archive staging, filtering,
  encryption, progress display, or forced overwrite.
