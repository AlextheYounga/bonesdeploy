# Tasks

## Implementation

- [x] Add `site import <archive> [--yes]` parsing and thin dispatch, including
  command tests for accepted and rejected forms.
- [x] Add the local site-import workflow with regular-file validation,
  exact-project confirmation, privileged streaming upload, SSH close handling,
  and success output that reports `.env` preservation and service restart.
- [x] Add fixed BonesRemote shared command builders and focused tests proving
  configured site identity is passed without accepting arbitrary remote paths
  or commands.
- [x] Add the BonesRemote shared command group, root checks, `SiteMutation`
  acquisition, idle-state enforcement for import, and command-dispatch tests.
- [x] Implement private same-filesystem transaction receipt with a filesystem
  safety reserve, durable transaction metadata, cleanup, and detection of an
  interrupted prior import.
- [x] Implement defensive ZIP validation and staged extraction for the exact
  `shared/` layout, including safe relative links, entry/path/space limits,
  duplicate and conflict rejection, `.env` omission, and ownership/mode
  normalization.
- [x] Extend the service lifecycle boundary to stop the project target, restart
  it, and verify registered services without introducing framework-specific
  behavior.
- [x] Implement environment carryover, durable cutover, atomic directory
  exchange, successful cleanup, restart rollback, and interrupted-transaction
  recovery while retaining both trees whenever recovery is incomplete.
- [x] Route `secrets push` through a typed BonesRemote environment operation
  using the same site mutation lock while preserving dotenv validation, atomic
  replacement, protected metadata, and existing CLI output.
- [x] Add focused tests for local preflight/confirmation/transfer behavior,
  hostile and valid ZIPs, normalized metadata, exact replacement, `.env`
  preservation, mutation serialization, cutover ordering, rollback failure, and
  interrupted import recovery.

## Validation

- [x] Run the focused BonesDeploy CLI, site-import, secrets, SSH, and command
  helper tests; confirm local failures do not contact or mutate the remote site.
- [x] Run focused BonesRemote archive and transaction tests; confirm hostile or
  malformed archives never reach cutover and successful imports remove stale
  live files while preserving `.env`.
- [x] Exercise simulated service-start, rollback, and process-interruption
  failures; confirm old and new trees remain recoverable and the next invocation
  converges to the previous live state before accepting input.
- [x] Run full non-E2E Rust and Python test suites, `cargo clippy`, `cargo fmt`,
  `shfmt -w .`, Ruff checks, and `git diff --check` with no warnings or failures.
- [x] Inspect `bonesdeploy site import --help` and BonesRemote shared-command help
  for accurate archive, confirmation, and environment-preservation wording.

## Completion

- [x] Document export-compatible and rsync-derived archive construction,
  replacement and downtime semantics, `.env` preservation, disk-space needs,
  rollback/recovery behavior, and the sensitivity of archive contents.
- [x] Document that volatile state is restored opaquely, including Laravel file
  sessions and caches, database consistency responsibility, and optional manual
  `php artisan optimize:clear` checks.
- [x] Update architecture, security, and context documentation for the typed
  shared mutation boundary and serialized environment publication where those
  documents describe the affected contracts.
- [x] Review the final diff for unsafe ZIP extraction, unbounded resource use,
  non-atomic replacement, environment races, framework-specific cleanup,
  obsolete direct secret mutation, scaffold text, and unrelated changes.

## Completion notes

- Implemented the planned typed shared mutation boundary, defensive ZIP
  extraction, same-filesystem atomic exchange, inode-based interruption
  recovery, verified service restart/rollback, and serialized environment
  publication.
- Export-compatible archives and manually built archives whose entries are all
  rooted beneath `shared/` are accepted; an explicit directory entry for
  `shared/` is not required.
- Focused tests cover malformed and hostile ZIPs, metadata normalization,
  environment omission/carryover ordering, atomic exchange, restart rollback,
  retained double-failure state, and interrupted post-exchange recovery. The
  existing SSH transport suite covers reader, remote-exit, timeout, and bounded
  diagnostic failures used by the local import orchestrator.
- Validation passed: `cargo test --workspace --exclude e2e`, workspace Clippy
  with all targets/features and warnings denied, `cargo fmt`, `shfmt -w .`, 508
  Python tests, Ruff checks/format verification, rendered CLI help, and
  `git diff --check`. E2E tests were deliberately not run.
