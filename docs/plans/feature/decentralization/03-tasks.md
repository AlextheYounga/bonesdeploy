# Project-Local Infrastructure Refactor Tasks

## Implementation

- [ ] Complete the `Bones` consumer audit and record each surviving value's
      destination as convention, local `.env`, `.env.build`, committed
      `infra/`, machine state, or obsolete.
- [ ] Replace shared `.bones` and `bones.toml` path/config APIs with explicit
      project identity, local `.env`, `.env.build`, `infra/`, and derived
      machine-path boundaries; remove fields classified obsolete.
- [x] Refactor `bonesdeploy init` and framework scaffolding to create ordinary
      committed `infra/` content without symlinks, nested Git, or config-root
      per-project copies, while preserving `.env.build`, framework asset
      behavior, and machine-local BonesDeploy data including the GPG keyring.
- [x] Scaffold project-facing provisioning under
      `infra/provision/core/` and `infra/provision/custom/`, mark core as
      BonesDeploy-managed, and compose core before custom through explicit
      Python execution.
- [x] Make all commands that depend on project loading fail clearly when they
       encounter an old `.bones` layout, directing the user to `bonesdeploy
       update`. No ordinary command detects, adapts to, or silently supports
       the old layout.
- [x] Refactor local provisioning, doctor, manifest, runtime, services, SSL,
      helpers, status, rollback, update, and CLI config/help flows to consume
      the new local inputs and machine conventions.
- [x] Remove `bonesdeploy push`, config-repository setup, site import/receive,
      infrastructure hooks, and deploy-on-push behavior from both binaries and
      their embedded assets.
- [x] Move the existing encrypted project secret path into `infra/secrets/`,
      preserve encrypted bytes and protected remote shared-state delivery, and
      ensure GPG private keys, keyrings, plaintext, and other decryption
      authority never enter Git or build execution.
- [x] Add the BonesRemote immutable revision boundary: resolve the full SHA,
      validate site identity, read revision-owned deployment files, and pass
      one snapshot through stage, checkout, build, prepare, seal, activate,
      rollback, and cleanup without mutable site configuration reloads.
- [x] Update BonesInfra context loading and project infrastructure execution
      to use local `.env` plus committed `infra/` behavior without introducing
      a replacement TOML configuration layer.
- [x] Remove `Runtime.shared`, `SharedPath`, and `SharedPathType`; make setup
      create `shared/.env` for every framework and make deployment wire that
      file into each release.
- [x] Add framework-owned lists of directory-valued environment variables and
      create only those directories during provisioning. Do not create
      application data files such as SQLite databases.
- [x] Implement the explicit BonesDeploy update path for managed provisioning:
      refresh unmodified `infra/provision/core/` files, preserve all custom
      files, and report modified-core conflicts without silently overwriting
      them or performing a three-way merge.
- [x] Restore the versioned Python update-patch registry and implement the
       `0.8.0` local patch that safely transitions `.bones` to `infra/`,
       preserves files and encrypted secret bytes, leaves machine-local GPG
       state untouched, refuses unsafe layouts, and does not create Git
       commits. `bonesdeploy update` is the single bridge; no ordinary command
       supports the old layout.
- [ ] Remove obsolete templates, fixtures, hooks, and code exposed by the
      refactor, keeping unrelated runtime permission behavior unchanged.

## Validation

- [ ] Add focused parser, identity, path, revision-consistency, and
      build-secret-boundary tests at the owning Rust/Python modules.
- [x] Add isolated CLI tests for fresh init, old-project migration, explicit
      deploy behavior, and rejection of obsolete transport/config commands.
- [ ] Add BonesRemote tests proving differing Git revisions cannot mix source,
      deployment scripts, infrastructure code, or scalar build inputs.
- [ ] Add regression tests proving encrypted secrets remain encrypted in Git,
      runtime plaintext is excluded from build inputs, secret delivery retains
      protected ownership and mode, and migration leaves local GPG keyrings
      untouched.
- [x] Add shared-state tests proving `.env` is created and wired universally,
      declared framework directories are created, and application data files
      are not created by BonesDeploy.
- [x] Run affected Rust and Python tests and inspect their observable results.
- [x] Run `cargo clippy` and address all warnings/errors.
- [x] Run `cargo fmt` and verify no formatting diff remains.
- [x] Run `shfmt -w .` and verify shell files remain valid.
- [x] Run the Python lint/format checks required by
      `crates/bonesinfra/python/AGENTS.md` without running e2e tests.

## Completion

- [x] Update `CONTEXT.md`, `README.md`, and related BonesInfra documentation
      so they describe project-local `infra/`, local provisioning inputs, and
      explicit deployment only.
- [ ] Search for stale `.bones`, `bones.toml`, config-repository,
      import/receive, and deploy-on-push references and remove or intentionally
      document every remaining internal compatibility reference.
- [ ] Review the final diff against `01-idea.md` and `02-plan.md`; record only
      meaningful deviations, validation results, discoveries, and deliberately
      unfinished excluded work in completion notes.

## Completion notes

- Shared-state implementation is complete: setup owns `shared/.env`, release
  wiring is unconditional, and framework runtimes declare directory-only
  shared paths.
- Removed `runtime.shared` configuration is rejected instead of silently
  ignored through runtime's flattened extension values.
- Focused shared-directory and release-wiring coverage exists; setup creation
  and application-data-file non-creation assertions are covered by the
  framework provisioning tests.
- The general-purpose `Bones` type remains as an internal compatibility-shaped
  value object for remote machine inputs; project-local persistence is now the
  flat root `.env`, and no command writes or loads a project `bones.toml`.
- The old `.bones` layout is transitioned only by the version-gated `0.8.0`
  update patch; ordinary commands reject it. E2E fixtures and historical plan
  references retain explanatory legacy examples and are not runtime
  compatibility paths.
- Restored the Python patch registry with the local `0003-project-infra` patch
  and marker-only remote completion. The patch's focused tests cover version
  selection, ciphertext-preserving migration, refusal without a marker, and
  remote marker creation.
- Validation passed: `uv run pytest` (379 tests), `cargo test -p bonesdeploy
  -p bonesinfra`, `cargo clippy`, `cargo fmt --check`, `ruff check .`, and
  `shfmt -d .`. E2E tests were not run.
- Full Python and Rust validation is recorded in the implementation session;
  e2e tests were intentionally not run.
