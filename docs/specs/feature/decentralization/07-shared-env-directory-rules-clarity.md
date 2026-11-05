# Clarification

<!--
This document records a settled change to the existing planning record.

Never use a clarification file to ask a question or preserve undecided
alternatives.
-->

## Trigger

The shared-path proposal was simplified: application files such as Laravel's
`database.sqlite` are created by the application, while BonesDeploy should only
create directories needed by runtime configuration, with `.env` as the one
managed file.

## Decision

Remove the configurable `runtime.shared` object and its `SharedPath` types.
Every setup creates `shared/` and `shared/.env` by default; `.env` remains the
only shared file that BonesDeploy creates and wires into releases.

BonesDeploy may create known framework directories referenced by local
environment values, such as Laravel's `LARAVEL_STORAGE_PATH`,
`VIEW_COMPILED_PATH`, `CACHE_PATH`, and `UPLOADS_PATH`. Framework modules define
which environment variables represent directories. BonesDeploy does not create
application data files such as `database.sqlite`; the application owns those
files.

The deployment path always links `shared/.env` into the active release as
`.env`. No generic shared-path allowlist or file/directory inference is used.
The initial directory creation uses the existing runtime ownership and a
consistent writable directory mode; framework-specific permission policy is
not introduced as part of this clarification.

## Supersedes

Supersedes the earlier proposal to infer both files and directories from every
path-like `.env` value and to add framework-specific file permission objects.
It adds a narrower rule to the existing decision to remove `runtime.shared`.

## Effect on the record

- `01-idea.md`: Defines `shared/.env` as the only managed shared file and
  application-owned data files as excluded from automatic creation.
- `02-plan.md`: Replaces generic shared-path inference with framework-declared
  directory creation and unconditional `.env` wiring.
- `03-tasks.md`: Adds removal of the shared object, default `.env` wiring, and
  directory-only creation behavior.
