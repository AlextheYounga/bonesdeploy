# Plan

## Current behavior

`e2e/tests/setup/harness.rs` calls `SampleProject::pin_node_version` for every
Next, Nuxt, SvelteKit, and Vue fixture after project initialization.
`e2e/src/project.rs` replaces only the literal `NODE_VERSION=\n` and errors
when that literal is absent. `bonesdeploy/src/frameworks.rs` now gives Next and
Nuxt a default Node version, which initialization writes as a non-empty
`NODE_VERSION=<version>` assignment. Next and Nuxt therefore stop before their
E2E site setup begins.

## Intended behavior

The E2E helper will replace the value of the existing `NODE_VERSION` assignment
with the requested E2E pin. It will reject a build environment that has no Node
version key, preventing a silent malformed fixture setup.

## Approach

Replace the helper's empty-line string replacement with line-aware replacement
of the `NODE_VERSION=` assignment. Keep all other `.env.build` content intact,
and add focused tests for empty and defaulted assignments plus a missing-key
error.

## Responsibilities and boundaries

`SampleProject` owns E2E fixture mutation and will own the replacement logic.
Framework initialization continues to own production `.env.build` defaults;
the E2E harness only overrides the value for its reproducible fixture builds.

## Affected areas

- `e2e/src/project.rs`: replace an existing Node-version assignment without
  assuming its value is empty.
- `e2e` tests: cover valid empty/defaulted values and a missing key.
- `docs/plans/fix/e2e-node-build-version/`: retain the defect and validation
  record.

## Decisions

- Match the key at the start of a line so only `NODE_VERSION` is changed and
  comments or similarly named variables remain unchanged.
- Keep a missing-key error because a fixture without build Node configuration is
  malformed and must not progress to an ambiguous build.
- Do not change framework defaults; they are valid production configuration and
  the defect is only the stale E2E helper assumption.

## Risks

- Broad replacement could alter comments or a differently named variable;
  line-aware matching prevents that.
- Replacing a missing key silently would hide a fixture-generation regression.
- The corrected setup can expose later host builder-image or deployment failures
  that were previously masked.

## Validation

- Run the focused E2E project-helper tests; confirm empty and defaulted Node
  values become the requested pin and a missing key fails.
- Run `cargo test -p e2e`.
- With the pinned Docker builder image available, run Next server/static and
  Nuxt server/static E2E scenarios serially.
- Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`, then inspect the final
  diff.
