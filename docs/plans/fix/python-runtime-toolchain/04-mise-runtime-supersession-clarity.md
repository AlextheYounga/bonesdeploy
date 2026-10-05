# Mise Runtime Supersession Clarification

## Trigger

`feature/mise-runtimes` establishes one exact Python runtime contract for local
builds and production.

## Decision

Django uses the exact `python_version` selected in Bones configuration. Pinned
mise `2026.10.0` installs that runtime from precompiled artifacts only in the
local build cache and root-owned production store. Production creates a stable
per-site Python link and services invoke it directly. Django artifacts package
application dependencies and site-link launchers; prepare validates runtime
identity before checks, migrations, or static collection and does not install
packages.

## Supersedes

The distribution Python, `/usr/bin/python3`, release-owned virtualenv, and
target-side pip installation decisions in this plan.

## Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` now identify
`feature/mise-runtimes` as the current authoritative runtime contract.
