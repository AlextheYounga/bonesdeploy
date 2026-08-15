# Clarification

## Trigger

Review of the completed refactor found that framework implementations were
moved from the BonesInfra Python package into `crates/bonesdeploy/assets/`.
This makes BonesInfra unable to provide its own framework support and inverts
ownership between the two crates.

## Decision

BonesInfra owns the canonical framework implementations and their scaffold
resources under its Python package. BonesDeploy owns project initialization and
requests BonesInfra to materialize the selected framework into `.bones/infra/`.

The materialized `.bones/infra/` package is a vendored project snapshot. Newly
initialized projects receive that snapshot and execute it by default, so the
infrastructure is readable, committed, editable, and stable across BonesInfra
updates.

Runtime and manifest loading use one strict precedence rule:

- When the complete `.bones/infra/` package is absent, load the selected built-in
  framework implementation from BonesInfra.
- When `.bones/infra/` exists, load the complete local package and never consult
  the built-in implementation for that project.
- A present but incomplete, syntactically invalid, import-invalid, or contract-
  invalid local package fails with a path-specific error; it never falls back.

The local package is all-or-nothing for runtime and manifest resolution. This
prevents a project runtime from being paired accidentally with a different
manifest implementation. The framework registry, implicit template fallback,
and root hook system remain removed.

## Supersedes

This replaces the earlier ownership decision that generated framework source is
canonical under `crates/bonesdeploy/assets/frameworks/` and that BonesInfra is
framework-independent. It also replaces the requirement that projects without
`.bones/infra/` fail rather than use a built-in framework implementation.

## Effect on the record

`01-idea.md` now defines canonical framework implementations, built-in
framework loading, and vendored project snapshots. `02-plan.md` now assigns
framework source and materialization to BonesInfra, assigns initialization
coordination to BonesDeploy, and defines strict local-package precedence.
`03-tasks.md` replaces the completed snapshot-only implementation tasks with
the concrete ownership correction, loader fallback, materialization, tests, and
documentation work required by this clarification.
