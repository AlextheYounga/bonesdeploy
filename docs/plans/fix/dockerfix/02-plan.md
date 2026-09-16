# Plan

## Current behavior

`RuntimeBackend` in `crates/bonesdeploy-core/src/config/model.rs` represents
`native` and `docker`. Local `.env` loading, provisioning transport, init
arguments, and Python request parsing carry that selection. Native remains the
serde and initialization default.

The shared deployment lifecycle in
`crates/bonesremote/src/commands/deploy/coordinator.rs` stages a release,
exports one resolved Git commit into a build context, runs numbered build
scripts through a rootless Podman build container, promotes the output, wires
shared paths, runs prepare scripts, seals the release, validates preflight,
atomically switches `current`, restarts site systemd services, verifies them,
and prunes old releases. `SiteMutation` and the centralized state store serialize
and record that lifecycle. Rollback switches `current` back and restarts the
same site services.

The Docker branch changes only Laravel. The Python module
`crates/bonesinfra/python/src/bonesinfra/frameworks/laravel/docker.py` installs
Docker, generates and builds a BonesDeploy-owned PHP-FPM image, renders Laravel
nginx and systemd configuration, and registers one Docker service. The Rust
module `crates/bonesremote/src/runtime/docker/` constructs a single `docker run`
command with Laravel-specific image and socket names and forces `php-fpm -F`.
Docker prepare scripts run in that generated runtime image. Project Dockerfiles
and Compose files are not used.

Built-in database and cache selection is stored in `Bones.services`, serialized
through `ServicesRequest`, populated from encrypted environment keys, and
provisioned by the Python registry under
`crates/bonesinfra/python/src/bonesinfra/services/runtime/`. This service model
is duplicated across Rust configuration, Rust secret handling, JSON transport,
Python request parsing, Python provisioners, shell templates, manifests, tests,
and user documentation.

Current security documentation treats project-controlled Compose definitions
as incompatible with a privileged controlled runtime. The Docker documentation
therefore promises fixed mounts, capabilities, identity, command, network, and
socket behavior that applies only to the Laravel-specific container.

## Intended behavior

Native deployment behavior remains unchanged. It continues to use rootless
Podman for numbered application build scripts, host-user prepare scripts, and
BonesDeploy-managed systemd runtimes.

For a Compose deployment, the candidate release contains the project-owned
Compose file, Dockerfiles, build contexts, and related source. The Docker
backend validates the candidate with `docker compose config --quiet`, pulls
referenced images, and runs `docker compose build` before activation. It does
not run the native rootless Podman application build scripts or the generated
Laravel prepare container.

After the candidate has been promoted, wired to `shared/.env`, and sealed, the
existing lifecycle atomically points `current` at it. The generic Compose site
service then reconciles the stable `bonesdeploy-<site>` project from `current`
with `docker compose up --detach --build --remove-orphans --wait`. Docker
Compose receives `shared/.env` as its environment file. A successful start
requires Compose to report the stack running or healthy; failure enters the
existing activation rollback path.

Rollback points `current` at the previous release and runs the same Compose
reconciliation against that release. Local images are rebuilt from the previous
Dockerfiles when necessary. Stable named volumes remain attached and their
contents are not rolled back. Release pruning removes old source releases only.

Compose files use Docker's conventional root-level names. BonesDeploy selects
exactly one base file in this order: `compose.yaml`, `compose.yml`,
`docker-compose.yaml`, then `docker-compose.yml`. More than one base file is an
error rather than an implicit precedence choice. It also includes exactly one
optional override, preferring `compose.override.yaml` over
`compose.override.yml`; both override files together are an error. Every
Compose command receives this explicit ordered file set, so `COMPOSE_FILE`
cannot redirect deployment outside the resolved release.

`BONES_COMPOSE_PORT` is an optional canonical `u16` configuration value with no
default. Values must be in `1..=65535`. When present, BonesInfra renders host
nginx to proxy to `127.0.0.1:<port>`, and doctor requires the stack to publish
that port on loopback. A Compose site with a configured domain, SSL, or managed
Quick Tunnel requires `BONES_COMPOSE_PORT`. Without it, the stack owns ingress
and may publish ports directly. Public bindings remain valid in either mode but
doctor reports that they bypass BonesDeploy-managed ingress.

The `services` configuration and command behavior are removed. Init no longer
offers host database/cache selection, secrets no longer generate service-owned
credentials, and BonesInfra no longer provisions those services. A Compose
stack declares supporting services using normal Compose syntax.

Compose setup and documentation state that the Compose file is trusted
privileged deployment input. BonesDeploy continues to protect its release and
control-plane state and does not add Docker socket mounts, but it does not
claim native isolation for user-selected container settings.

## Approach

Keep one deployment state machine and branch only at the existing build,
runtime provisioning, preflight, and service execution boundaries.

Replace the existing Laravel-specific `bonesremote::runtime::docker` behavior
with a generic Compose runtime boundary. It resolves the Compose file beneath a
specified release root, constructs Docker Compose commands as argument vectors,
sets the stable project name and project directory, supplies the protected
environment file for interpolation, performs candidate validation/pull/build,
starts and stops the active stack, and inspects Compose state without rendering
secret values. The runtime clears inherited Compose control variables and uses
explicit `--file`, `--project-name`, `--project-directory`, and `--env-file`
arguments for every operation.

The build context produced by checkout is the project directory for candidate
`config --quiet`, `pull`, and `build` operations. Promotion copies that same
validated source tree into the versioned release. Activation always runs
`up --detach --build --remove-orphans --wait` from `current`; the second build
uses Docker cache but guarantees that active and rollback starts use the
Dockerfiles belonging to the selected release.

During the build phase, native sites retain `build::run`. Compose sites invoke
the Compose candidate preparation operation instead. Promotion, shared wiring,
sealing, activation, state transitions, and cleanup remain common. Compose does
not execute numbered prepare scripts; stack-specific initialization and
migrations belong in the Compose services, images, entrypoints, and health
checks.

Replace Laravel Docker provisioning with one generic Compose backend in
BonesInfra. It configures Docker's supported apt repository for the detected
Debian or Ubuntu release, installs Docker Engine and the Compose plugin from
that repository, renders a generic
oneshot systemd service with `RemainAfterExit`, registers it in the existing
site target, and optionally renders nginx for a configured loopback ingress
port. `ExecStart` delegates to root-only `bonesremote runtime start`; `ExecStop`
delegates to `bonesremote runtime stop`. Starting uses detached Compose with
readiness waiting. Stopping uses `docker compose stop` and never deletes named
volumes.

Docker backend dispatch occurs before framework runtime provisioning. A Docker
site receives the generic Compose unit and optional nginx/tunnel units, but no
framework-managed application unit, PHP-FPM pool, worker, runtime image, or
framework runtime manifest entry. Project-owned custom infrastructure remains
composed after the generic backend. Provisioning removes and unregisters known
framework runtime units and Laravel Docker artifacts left by an earlier backend
selection before registering the Compose unit.

Compose-only doctor runs skip the native Podman availability check while
server-wide and native-site doctor runs retain it. Compose service-state and
ingress findings are classified through a pure evaluator so warning and failure
mapping can be tested without invoking Docker.

The Compose manifest reports an explicit inspection error when `bonesremote
status` is unavailable or empty, preserves status-reported Compose errors, and
continues to report Docker Engine and plugin availability independently.

Delete the built-in service model end to end rather than retaining dead
configuration compatibility. Remove service selection from init, the canonical
config and provisioning transport fields, generated database/cache secrets,
the site services command, Python service request parsing and registry,
provisioners and templates, manifest entries, tests, and documentation.

Update doctor and manifest around observable Compose behavior: installed
engine/plugin, discoverable and valid Compose file, generic systemd unit and
target registration, stable project identity, running/healthy containers, and
optional ingress reachability. Doctor emits one explicit reduced-guarantee
warning for Compose sites; it does not attempt to certify arbitrary Compose
container security.

`BONES_COMPOSE_WAIT_TIMEOUT` is a canonical integer configuration value with a
default of 120 seconds and a valid range of 1 through 3600. Runtime start passes
it to `docker compose up --wait-timeout`. The Compose command's zero exit status
after `--wait` is the deployment readiness decision: services with health
checks must be healthy and services without them must be running according to
Docker Compose. Status reports each container's actual running, healthy,
unhealthy, or exited state. Doctor fails unhealthy and nonzero-exited services,
reports zero-exited one-shot services as completed, and warns that a running
service without a health check has not proven application readiness.

The protected `shared/.env` is an interpolation source, not automatic container
environment injection. Projects explicitly reference values with Compose
`environment` entries or use `env_file: .env`, which resolves through the
release's existing `.env` link to `shared/.env`. BonesDeploy uses only quiet
configuration validation and never prints rendered Compose configuration.
Docker build output and application logs remain project-controlled and may
expose values that the project deliberately prints.

## Responsibilities and boundaries

`bonesdeploy-core` owns backend representation, `BONES_COMPOSE_PORT`,
`BONES_COMPOSE_WAIT_TIMEOUT`, and their transport and validation. It no longer
owns built-in service selection or credential transport.

`bonesdeploy` owns init selection, local validation, encrypted application
secrets, control-plane synchronization, and user-facing commands. It does not
interpret Compose files or generate service credentials.

Docker Compose owns parsing and interpolation of Compose files, image pull and
build behavior, service dependencies, networks, volumes, and container
lifecycle semantics.

BonesInfra owns host provisioning: Docker Engine and Compose installation,
the generic site systemd unit, target registration, optional nginx ingress,
and provisioning-time manifest declarations. Framework packages no longer own
Docker behavior.

BonesRemote owns privileged deployment-time Compose operations and inspection.
The existing deployment lifecycle remains the sole owner of release state,
locking, promotion, activation, rollback, and pruning.

The project owns its Compose file, Dockerfiles, container security, supporting
services, persistent data declarations, service health checks, and any ports it
publishes outside BonesDeploy-managed ingress.

## Affected areas

- `crates/bonesdeploy-core/src/config/` and its tests: remove services and add
  canonical Compose ingress configuration and transport.
- `crates/bonesdeploy/src/cli/`, `commands/init/`, `commands/site/`,
  `commands/secrets/`, framework assets, and tests: remove service selection and
  generated credentials; describe Docker as Compose.
- `crates/bonesremote/src/commands/deploy/`, `release/lifecycle/`, `runtime/`,
  `commands/service.rs`, `commands/doctor/`, `commands/status.rs`, and tests:
  replace fixed Laravel container operations with candidate and active Compose
  operations.
- `crates/bonesinfra/python/src/bonesinfra/config/`, `cli/commands/site/`,
  `services/runtime/`, framework runtime and manifest modules, Docker/systemd/
  nginx assets, and Python tests: remove built-in services and provision the
  generic Compose backend.
- Embedded skill documents, `README.md`, `CONTEXT.md`,
  `crates/bonesinfra/python/CONTEXT.md`, `docs/ARCHITECTURE.md`, architecture
  references, and security invariants: record the new backend and remove
  built-in-service claims.
- `e2e/` fixtures and ignored tests: define representative Compose stacks and
  lifecycle assertions without running them locally.

## Decisions

- Rootful Docker Compose is the chosen backend because compatibility with
  existing stacks and lean delegation are explicit product requirements.
- `docker` means a general Compose deployment after this change. The old
  generated Laravel runtime is removed rather than retained under another name.
- Compose is framework-independent. Framework modules do not branch on the
  Docker backend or generate container images and commands.
- Native and Compose use different application build mechanisms. Native keeps
  rootless Podman build scripts; Compose uses `docker compose pull` and
  `docker compose build`.
- Compose initialization and migrations are project-owned container behavior.
  Numbered host prepare scripts are not run for Compose deployments.
- The Compose project name remains stable across releases so named volumes and
  service DNS identities persist. Filesystem release identity remains owned by
  BonesRemote rather than becoming the Compose project name.
- Compose definitions are accepted as privileged deployment instructions.
  BonesDeploy warns about reduced guarantees but does not implement a partial
  Compose parser or compatibility-breaking security allowlist.
- No Docker socket is injected automatically. A project that explicitly mounts
  it has deliberately left BonesDeploy's guaranteed security boundary.
- Host nginx integration is optional and uses one configured loopback ingress
  port. Stacks remain free to own public port exposure directly.
- Compose file discovery is deterministic and does not honor `COMPOSE_FILE`.
  One conventional base file and at most one conventional override file keep
  all privileged definitions inside the selected release.
- `docker compose up --wait` is the activation readiness authority. The fixed
  configurable timeout prevents a stack from wedging the deployment lock.
- Built-in service configuration is removed without backward-compatibility
  shims because the feature has never provided a reliable shipped contract.
- Persistent Compose data is Docker-owned. Release rollback and pruning never
  delete or rewind named volumes.
- Docker Engine and Compose packages come from Docker's official apt repository
  for supported Debian and Ubuntu releases; BonesInfra rejects any other
  distribution instead of combining incompatible distribution and upstream
  package names.

## Risks

- A project-controlled Compose file can grant containers broad host authority
  through privileged mode, host mounts, devices, capabilities, namespaces, or
  public ports. Setup and doctor must identify Compose as reduced-guarantee
  mode, and documentation must not repeat native security claims for it.
- Rootful Compose builds and runtime operations execute repository-controlled
  instructions through a privileged daemon. Only the trusted project operator
  may authorize and trigger deployments.
- Compose stacks without health checks may be considered ready when containers
  are merely running. Documentation and diagnostics must distinguish running
  from healthy services.
- A failed activation may leave partially recreated containers before the
  existing rollback starts the previous stack. Compose errors and rollback
  errors must remain visible in persisted deployment state.
- Stable named volumes preserve data across releases but also preserve
  incompatible schema changes. Rollback documentation must state that data is
  not rolled back.
- Locally built Compose images use mutable project/service tags. Rebuilding the
  previous release during rollback is necessary to restore its image contents.
- Removing built-in service keys changes existing project configuration and
  encrypted-environment scaffolding. Errors must clearly direct affected users
  to native external services or Compose definitions.
- Compose-managed public ports can bypass nginx, TLS, UFW assumptions, and
  per-site ingress isolation. Doctor must report this reduced visibility
  without claiming the stack is secure.
- Docker and Compose package names and plugin availability vary across supported
  distributions. Provisioning and doctor must validate the actual
  `docker compose` command rather than infer support from package installation.

## Validation

- Core and CLI tests prove omitted backends remain native, Docker selects the
  Compose path for every framework/custom project, Compose ingress values are
  validated, wait timeout values are bounded, and built-in service flags and
  configuration are rejected or absent.
- BonesRemote tests execute a fake Docker command boundary and prove candidate
  Compose discovery and duplicate-file rejection, base-plus-override ordering,
  ignored `COMPOSE_FILE`, quiet validation, pull/build ordering, stable project
  naming, protected interpolation environment selection, active-stack
  start/stop, readiness timeout, failure propagation, and no automatic Docker
  socket mount.
- Lifecycle tests prove native builds still use rootless Podman, Compose builds
  do not use that pipeline, activation runs the stack from `current`, Compose
  startup failure restores the previous release, and release pruning does not
  issue volume-deletion commands.
- Python tests prove Compose sites install and verify Docker plus Compose,
  receive the generic systemd unit and optional nginx configuration, and bypass
  every framework-native runtime provisioner and manifest while preserving
  custom infrastructure. Migration tests prove stale framework units and the
  former Laravel Docker artifacts are removed. Native provisioning remains
  unchanged.
- Doctor, status, and manifest tests prove Compose file, engine, plugin, unit,
  container state, health, ingress, and reduced-guarantee reporting without
  exposing resolved environment values.
- Focused fake-command tests prove candidate operation order, start and stop
  arguments, timeout handling, first-failure propagation, and absence of volume
  deletion without requiring Docker. Existing rollback tests prove that failed
  activation restores and restarts the previous release definition.
- Service-removal tests and searches prove the CLI, canonical config, transport,
  Python registry, provisioners, templates, and generated docs no longer expose
  PostgreSQL, MySQL, MariaDB, MongoDB, Redis, or Valkey as built-in services.
- Ignored E2E definitions cover a web-plus-database Compose stack, multiple
  services and networks, a worker, a custom non-Laravel project, a project-owned
  Dockerfile, base-plus-override behavior, named-volume persistence across two
  deployments, health-gated startup, failed activation rollback, direct public
  ports, and optional nginx ingress. They are reviewed but not executed locally.
- Run affected Rust tests and the full non-E2E workspace tests, Python tests,
  `cargo clippy`, `cargo fmt`, `shfmt -w .`, `ruff check .`, and `ruff format .`.
  Review the final diff for obsolete Laravel Docker code, built-in service
  remnants, secret output, volume deletion, accidental native-path changes,
  and security claims that fail to distinguish native and Compose modes.
