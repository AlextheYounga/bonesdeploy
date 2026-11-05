# Clarification

## Trigger

The framework investigation found more boundary defects than the parent summary
listed. The concrete findings must remain available to the Framework child plan.

## Decision

The Rust Framework child change owns all Rust-side framework selection behavior,
including defaults and asset selection, not only questions and environment
examples.

The current framework boundary includes:

- `crates/bonesdeploy/src/frameworks.rs` string dispatch for questions,
  validation, configuration, environment examples, and build examples;
- `infra/assets/frameworks.rs::framework_defaults()` as an additional dispatch
  path that reaches per-framework defaults directly;
- `FrameworkDefaults` and `PermissionDefault` as part of the Rust framework
  contract;
- centralized `frameworks.rs::validate_answers()` driven by question schemas,
  rather than independent per-framework validators;
- `Runtime.template: String`, `FrameworkSelection.template: Option<String>`,
  embedded asset directories, and Python `BUILTIN_FRAMEWORKS` as separate identity
  sources;
- eight wire identities, including `custom`;
- Python materialization into `infra/.framework` and `infra/custom`,
  with managed core and user-owned custom content.

The child plan must close the defaults/asset reach-through, define the typed Rust
front door, preserve the `.env` wire contract, and test Rust/Python/materialization
consistency. It must not move Python provisioning ownership into Rust.

## Supersedes

This adds concrete framework findings to the parent architecture inventory. It
supersedes the narrower assumption that framework work only covered five dispatch
functions.

## Effect on the record

- `02-plan.md` identifies framework defaults, centralized validation, eight wire
  identities, and the materialization boundary as part of the current behavior.
- `03-tasks.md` requires the Framework child plan to cover defaults, assets,
  identity registries, and cross-layer materialization tests.
