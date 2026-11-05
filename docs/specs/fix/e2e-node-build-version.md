# E2E Node Build Version

<!-- Merged from fix/e2e-node-build-version -->

## Idea

### Request

Record and repair the Next and Nuxt native E2E setup error on a dedicated Git
Flow bugfix branch.

### Problem

Next and Nuxt E2E scenarios fail before provisioning because
`SampleProject::pin_node_version` requires `.env.build` to contain an empty
`NODE_VERSION=` line. Current framework initialization writes its default Node
version instead, so the helper reports an error even though the generated build
environment is valid.

### Definitions

**Generated Node version:** The valid `NODE_VERSION` value written to
`.env.build` during Next or Nuxt project initialization.

**E2E Node pin:** The explicit Node version required by the native E2E harness
to make its fixture builds reproducible. It replaces, rather than requires the
absence of, the generated Node version.

### Desired outcome

Next and Nuxt E2E setup writes the E2E Node pin into the generated `.env.build`
whether its existing `NODE_VERSION` value is empty or defaulted. The helper
still fails clearly when that key is absent.

### Scope

- Update the native E2E project helper to replace a generated `NODE_VERSION`
  assignment.
- Add regression coverage for empty, defaulted, and missing Node version keys.
- Re-run Next and Nuxt native E2E scenarios after the local Docker builder image
  prerequisite is available.

### Constraints

- Preserve the current framework-generated Node defaults outside the E2E helper.
- Do not change the E2E pin version or general `.env.build` parsing behavior.
- Use a Git Flow bugfix branch with a linked worktree.

### Exclusions

- Pulling the pinned `buildpack-deps` image or installing Docker emulation is a
  local host prerequisite, not a repository code change.
- Laravel's builder-image failure and the Compose placeholder doctor failure are
  separate issues.


## Plan

### Current behavior

`e2e/tests/setup/harness.rs` calls `SampleProject::pin_node_version` for every
Next, Nuxt, SvelteKit, and Vue fixture after project initialization.
`e2e/src/project.rs` replaces only the literal `NODE_VERSION=\n` and errors
when that literal is absent. `bonesdeploy/src/frameworks.rs` now gives Next and
Nuxt a default Node version, which initialization writes as a non-empty
`NODE_VERSION=<version>` assignment. Next and Nuxt therefore stop before their
E2E site setup begins.

### Intended behavior

The E2E helper will replace the value of the existing `NODE_VERSION` assignment
with the requested E2E pin. It will reject a build environment that has no Node
version key, preventing a silent malformed fixture setup.

### Approach

Replace the helper's empty-line string replacement with line-aware replacement
of the `NODE_VERSION=` assignment. Keep all other `.env.build` content intact,
and add focused tests for empty and defaulted assignments plus a missing-key
error.

### Responsibilities and boundaries

`SampleProject` owns E2E fixture mutation and will own the replacement logic.
Framework initialization continues to own production `.env.build` defaults;
the E2E harness only overrides the value for its reproducible fixture builds.

### Affected areas

- `e2e/src/project.rs`: replace an existing Node-version assignment without
  assuming its value is empty.
- `e2e` tests: cover valid empty/defaulted values and a missing key.
- `docs/plans/fix/e2e-node-build-version/`: retain the defect and validation
  record.

### Decisions

- Match the key at the start of a line so only `NODE_VERSION` is changed and
  comments or similarly named variables remain unchanged.
- Keep a missing-key error because a fixture without build Node configuration is
  malformed and must not progress to an ambiguous build.
- Do not change framework defaults; they are valid production configuration and
  the defect is only the stale E2E helper assumption.

### Risks

- Broad replacement could alter comments or a differently named variable;
  line-aware matching prevents that.
- Replacing a missing key silently would hide a fixture-generation regression.
- The corrected setup can expose later host builder-image or deployment failures
  that were previously masked.

### Validation

- Run the focused E2E project-helper tests; confirm empty and defaulted Node
  values become the requested pin and a missing key fails.
- Run `cargo test -p e2e`.
- With the pinned Docker builder image available, run Next server/static and
  Nuxt server/static E2E scenarios serially.
- Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`, then inspect the final
  diff.


## Tasks

### Implementation

- [x] Replace an existing `NODE_VERSION` assignment in the E2E project helper
  without requiring an empty value.
- [x] Add focused tests for empty, defaulted, and missing Node-version entries.

### Validation

- [x] Run the E2E project-helper tests and `cargo test -p e2e`.
- [ ] After the local pinned builder image is available, run Next server/static
  and Nuxt server/static E2E scenarios serially. Not run per the task constraint
  against E2E scenarios and ignored Incus-backed tests.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without unresolved
  warnings or formatting changes.

### Completion

- [x] Review the final diff to confirm it contains only the helper repair,
  regression tests, and this durable plan.

### Completion notes

On 2026-09-29, the native E2E batch reached Next and Nuxt after Laravel failed
on a missing local builder image. Both Next and Nuxt failed before provisioning
because their generated `.env.build` files contained defaulted Node versions
rather than the empty assignment the E2E helper expected.

Implemented `SampleProject::pin_node_version` with line-aware replacement of the
existing `NODE_VERSION=` assignment. Other `.env.build` content is preserved,
and a missing assignment still returns an error without writing the file.
Added three focused unit tests covering empty, defaulted, and missing keys.

Validation completed:

- `cargo test -p e2e --lib project::tests`
- `cargo test -p e2e`
- `cargo test -p e2e --test fixture_consistency`
- `cargo fmt --all -- --check`
- `cargo clippy -p e2e --all-targets --all-features -- -D warnings`
- `shfmt -w .`

Next/Nuxt Incus-backed E2E scenarios remain unrun as explicitly requested.
