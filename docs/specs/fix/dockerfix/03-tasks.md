# Tasks

## Implementation

- [x] Remove `Bones.services`, built-in service credential transport, local
  service selection, generated database/cache secrets, and the site services
  command; update Rust configuration and CLI tests so no built-in service API
  remains.
- [x] Remove Python built-in service request fields, validation, registry,
  provisioners, shell templates, command dispatch, manifest declarations, and
  tests while preserving unrelated language and Linux service modules.
- [x] Add and validate optional `BONES_COMPOSE_PORT` and bounded
  `BONES_COMPOSE_WAIT_TIMEOUT` configuration in `bonesdeploy-core`, local `.env`
  serialization, provisioning transport, init, and Python request parsing while
  retaining native as the default backend.
- [x] Replace Laravel-specific Docker provisioning with a generic Compose
  backend that installs and verifies Docker Engine plus the Compose plugin,
  renders the site Compose systemd unit, registers it in the site target, and
  renders optional loopback nginx ingress.
- [x] Delete the generated Laravel runtime Containerfile, PHP-FPM Docker
  configuration, image archive handoff, Docker framework branches, and
  Laravel-specific Docker manifest declarations after the generic backend owns
  site provisioning.
- [x] Route Docker provisioning before framework runtime behavior, preserve
  project-owned custom infrastructure, and remove or unregister stale managed
  framework units, PHP-FPM configuration, workers, and former Laravel Docker
  artifacts when a site changes to Compose.
- [x] Replace `bonesremote::runtime::docker` with generic Compose command and
  inspection behavior that deterministically selects one conventional base
  file and at most one override under a release root, rejects ambiguous file
  sets, clears inherited Compose control variables, and always supplies the
  explicit files, stable project name, project directory, and protected
  `shared/.env` interpolation path.
- [x] Route Compose candidates through quiet configuration validation, image
  pull, and Docker Compose build while retaining the rootless Podman numbered
  build-script path exclusively for native deployments.
- [x] Reconcile the active stack from `current` through the generic systemd
  service using detached `docker compose up --build --remove-orphans --wait`,
  enforce the configured wait timeout, stop it without deleting named volumes,
  and preserve existing target-based activation and rollback orchestration.
- [x] Skip numbered prepare scripts for Compose deployments and retain native
  host-user prepare behavior without changing shared wiring, release sealing,
  phase persistence, or pruning.
- [x] Extend Compose preflight, service verification, status, doctor, and
  manifest output with engine/plugin availability, Compose-file validity,
  systemd registration, container running/health state, optional ingress, and
  one explicit reduced-guarantee warning without printing resolved secrets;
  distinguish healthy, running without a health check, completed one-shot,
  unhealthy, and failed containers.
- [x] Replace Docker and built-in-service documentation in embedded skill docs,
  README, context files, architecture references, and security invariants with
  the native/Compose contract, privileged Compose-input warning, persistent
  volume behavior, and data-rollback limitation.
- [x] Replace the ignored Laravel-only Docker E2E definition with representative
  general Compose stacks covering a custom non-Laravel web service, project
  Dockerfile, database, worker, multiple services and networks, base plus
  override files, health checks, direct ports, optional nginx ingress, and
  persistent named volumes.

## Validation

- [x] Run focused Rust Core and CLI tests proving native defaulting, Compose
  configuration, framework-independent selection, and complete built-in service
  removal.
- [x] Run focused BonesRemote tests proving Compose command construction,
  deterministic discovery, duplicate-file rejection, ignored `COMPOSE_FILE`,
  base-plus-override ordering, validation/pull/build order, activation,
  readiness timeout, stop behavior, rollback, error propagation, stable project
  identity, and absence of volume deletion or automatic Docker socket mounts.
- [x] Run focused Python tests proving generic Compose provisioning and optional
  nginx ingress while every native framework retains its existing runtime path,
  Docker sites bypass framework runtime artifacts, custom infrastructure still
  runs, and stale managed runtime artifacts are removed.
- [x] Run doctor, status, manifest, and documentation checks proving reduced
  guarantees and runtime health are reported without secret values.
- [x] Review the ignored Compose E2E definitions without executing them locally
  and confirm they assert deployment, second deployment, volume persistence,
  health-gated startup, and rollback behavior.
- [x] Run the full non-E2E Rust and Python test suites and resolve every failure.
- [x] Run `cargo clippy`, `cargo fmt`, `shfmt -w .`, `ruff check .`, and
  `ruff format .`, resolving every warning or formatting change.

## Completion

- [x] Search for and remove obsolete Laravel image names, forced `php-fpm -F`,
  built-in service vocabulary, service credentials, host service provisioners,
  and claims that Docker does not execute Compose files.
- [x] Review the final diff for accidental native lifecycle changes, secret
  disclosure, Compose volume deletion, mutable checkout use, a second deployment
  state machine, and security language that overstates Compose guarantees.
- [x] Record implementation deviations, validation results, and deliberately
  unfinished work in the completion notes.

## Completion notes

Implemented the planned generic Docker Compose backend and removed the built-in
database/cache subsystem. Compose runtime inspection was centralized in
BonesRemote and reused by status, doctor, and the BonesInfra manifest. Compose
commands also clear Docker daemon-selection environment variables so privileged
operations cannot be redirected away from the local rootful daemon.

The ignored Compose E2E scenario uses a custom Dockerfile, base and override
files, web/database/worker services, two networks, direct and managed ingress,
health checks, and persistent named volumes. It defines second-deployment,
volume-persistence, and failed-activation rollback assertions. Per repository
policy it was compiled and reviewed but not executed locally.

Validation completed successfully:

- `cargo test --workspace --exclude e2e`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo test -p e2e --no-run`
- `cargo fmt --all -- --check`
- `uv run pytest` (`474 passed`)
- `uv run ruff check .`
- `uv run ruff format . --check`
- `shfmt -w .` and `shfmt -d .`
- `git diff --check`

The embedded BonesInfra wheel was regenerated after the final Python changes.
The follow-up integration audit identified additional hardening work, tracked
below. Execution of the ignored E2E scenario remains deferred to the dedicated
E2E environment.

## Integration hardening

- [x] Add a fake Compose command executor and focused tests for candidate
  validation/pull/build order, start and stop arguments, timeout handling,
  failure propagation, and the prohibition on volume deletion.
- [x] Skip the Podman availability check for Compose-only site doctor runs while
  retaining it for native sites, unknown site configuration, and server-wide
  diagnostics.
- [x] Extract and test Compose doctor service, health, ingress, and public-port
  finding classification without invoking Docker.
- [x] Install Docker Engine, Buildx, and Compose from Docker's supported Debian
  or Ubuntu apt repository and test repository selection and provisioning
  operations.
- [x] Extend native-to-Compose cleanup and tests to cover actual `gunicorn` and
  `puma` units, requirement links, AppArmor profiles, PHP-FPM configuration, and
  former Laravel Docker runtime artifacts.
- [x] Report and test manifest inspection failures for unavailable or invalid
  `bonesremote status`, unavailable Docker Engine, unavailable Compose plugin,
  and status-reported Compose errors.
- [x] Rebuild the embedded BonesInfra wheel and run focused tests, full non-E2E
  Rust and Python suites, clippy, Rust/Python/shell formatters, and final diff
  checks. Compile but do not execute the ignored E2E suite.

Hardening validation completed successfully:

- Fake-executor tests cover `config --quiet`, `pull`, `build`, active `up`,
  timeout arguments, `stop`, first-failure propagation, and forbidden volume
  deletion.
- Doctor tests cover Podman gating and all Compose service/ingress finding
  classes without invoking Docker.
- Python provisioning tests cover official Debian and Ubuntu repository
  selection, conflicting package removal, Compose package installation,
  `gunicorn`/`puma` migration cleanup, and former Laravel runtime artifacts.
- Manifest tests cover missing Docker, missing Compose, unavailable or invalid
  `bonesremote status`, and status-reported Compose errors.
- `cargo test --workspace --exclude e2e` passed.
- `cargo test -p e2e --no-run` passed; E2E tests were not executed.
- `cargo clippy --workspace --all-targets --all-features` passed.
- `uv run pytest` passed with 479 tests.
- Rust, Python, and shell formatter checks passed.

The embedded wheel was rebuilt after the final Python changes. No hardening work
is deliberately unfinished; only execution of the ignored E2E scenario remains
deferred to the dedicated E2E environment.
