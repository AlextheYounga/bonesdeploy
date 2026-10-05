# Mise Runtime Supersession Clarification

## Trigger

`feature/mise-runtimes` establishes one exact Ruby runtime contract for local
builds and production.

## Decision

Rails uses the exact `ruby_version` selected in Bones configuration. Pinned mise
`2026.10.0` installs that runtime from precompiled artifacts only in the local
build cache and root-owned production store. Production creates a stable per-site
Ruby link and services invoke it directly. Rails artifacts package
`vendor/bundle` and the lockfile-selected Bundler; prepare validates runtime
identity before migrations and does not install or compile gems.

## Supersedes

The distribution Ruby, `/usr/bin/ruby`, `/usr/bin/bundle`, and target-side bundle
installation decisions in this plan, including
`04-distribution-ruby-clarity.md`.

## Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` now identify
`feature/mise-runtimes` as the current authoritative runtime contract.
