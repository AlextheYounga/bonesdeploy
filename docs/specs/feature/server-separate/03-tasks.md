# Tasks

## Implementation

- [x] Add `ServerContext`, reshape `DeployContext` around its server
  connection, and generalize the pyinfra runner so server provisioning cannot
  access site configuration.
- [x] Replace the Python setup package with server and site operation packages;
  move global deploy-user behavior to server, per-site runtime/build identities
  to site, and shared image-store behavior to `services/linux/image_store.py`.
- [x] Implement explicit, idempotent `deploy_server_setup()` and
  `deploy_site_setup()` call sequences and expose them as `bonesinfra server
  apply` and `bonesinfra site apply`.
- [x] Extend BonesRemote host doctor and security checks for global roots,
  deploy identity, sudoers, image store, installed binary, supported platform,
  and host security services.
- [x] Replace the flat and remote Rust CLI variants with `server` and `site`
  hierarchies; retain root `setup` and `doctor` as thin composition commands;
  move project-scoped status, manifest, and releases under site; delete removed
  command implementations and guide compatibility code.
- [x] Implement server setup/apply/doctor and server helpers command flows;
  implement the site setup readiness guard and exact base, services, runtime,
  doctor sequence; make root setup delegate to server setup and then site setup;
  make root doctor run and aggregate server doctor plus site doctor; retain
  independent site runtime, services, and SSL flows.
- [x] Split skill readiness into server, site, and SSL checks and emit the
  required next command for uninitialized, server-missing, site-missing,
  SSL-missing, and ready states.
- [x] Update the shared-server E2E harness to execute server setup once and
  site setup for each framework project without running the E2E suite.
- [x] Replace old command references in user documentation, architecture and
  security documents, prompts and status guidance, `CONTEXT.md`, and embedded
  skill documents; document the exact non-deploying site setup sequence.

## Validation

- [x] Add focused Python tests for context separation, runner connection
  handling, server/site orchestrator call order, operation exclusion, and
  idempotence.
- [x] Add BonesRemote doctor tests for complete server-baseline evidence and
  retain site doctor coverage for site-specific evidence.
- [x] Add `bonesdeploy` integration tests for new parser/help routes, rejection
  of removed routes, root doctor composition and aggregation, `site
  doctor --local`, server-missing site setup failure, and the split skill state
  machine.
- [x] Update E2E harness-level tests or source assertions for the one-time
  server setup and per-project site setup lifecycle; do not execute E2E tests.
- [x] Run affected Python and Rust unit/integration suites, `cargo fmt`, `cargo
  clippy`, `shfmt -w .`, `ruff format .`, and `ruff check .`; resolve every
  warning and error.

## Completion

- [x] Review the final diff and generated public help to confirm root `setup`
  and `doctor` are only composition workflows, no obsolete flat, `remote`, or
  `guide` command remains, and no server flow reads site configuration.
- [x] Verify documentation and embedded guidance use `server` and `site`
  commands, and that every documented `site setup` sequence is readiness,
  base, services, runtime, doctor without Git, secrets, SSL, or deployment.

## Completion notes

Completed the server/site provisioning split, including isolated Python command
packages, complete host-mode BonesRemote baseline checks, scoped CLI coverage,
one-time shared-server E2E setup, and command-reference documentation updates.
The E2E target was compiled but its ignored scenarios were not executed.
