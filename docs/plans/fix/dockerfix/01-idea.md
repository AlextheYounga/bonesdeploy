# Idea

## Request

Replace the current Laravel-only Docker implementation with a general Docker
Compose deployment backend. A Docker deployment must run the vast majority of
ordinary Compose stacks, including stacks that define databases, caches,
workers, networks, volumes, and project-owned Dockerfiles. BonesDeploy must
remain lean, retain its release lifecycle, and explain that user-controlled
Compose deployments can bypass security measures that BonesDeploy guarantees
for native deployments.

Remove BonesDeploy's built-in database and cache services. Native deployments
may use independently managed services; Docker deployments define supporting
services in their Compose stack.

## Problem

The current `docker` backend is not a general Docker deployment. Only Laravel
provisioning uses it. BonesDeploy builds a fixed PHP-FPM runtime image, ignores
the project's Dockerfile and Compose files, starts one hard-coded container,
and forces `php-fpm -F`. The feature name and documentation imply flexibility
that the implementation does not provide.

The separate built-in service subsystem advertises PostgreSQL, MySQL, MariaDB,
MongoDB, Redis, and Valkey, but its configuration, credential transport,
provisioning, and health contracts have drifted and the services have not been
reliable. Maintaining host-native installation logic for every service and
distribution duplicates behavior already provided by mature Compose images.

## Definitions

**Native deployment:** A deployment whose application processes run in
BonesDeploy-managed host systemd services. Native deployments retain
BonesDeploy's managed runtime, filesystem, process, and service security
guarantees.

**Compose deployment:** A deployment whose image builds and runtime topology
are defined by a project-owned Compose file and executed by the conventional
rootful Docker Engine and Docker Compose plugin. The Compose file is trusted
privileged deployment input.

**Compose stack:** The services, builds, images, networks, volumes, health
checks, dependencies, and runtime options resolved by `docker compose` for one
site. BonesDeploy delegates their semantics to Docker Compose rather than
parsing or translating them.

**Compose project:** The stable Docker Compose identity
`bonesdeploy-<site>`. It remains constant across releases so Compose-managed
networks and named volumes persist while the active release changes.

**Reduced-guarantee mode:** The security contract for Compose deployments.
BonesDeploy continues to secure its own control plane, releases, secrets,
locking, and privileged entry points, but does not guarantee the safety of
project-selected images, mounts, capabilities, namespaces, ports, networks, or
container users.

**Built-in service:** The existing BonesDeploy feature that selects and
provisions a host database or cache through `BONES_SERVICES`, typed credential
transport, and BonesInfra runtime service implementations. This does not refer
to a service declared in a Compose file.

## Desired outcome

A user selecting the Docker backend can commit a conventional Compose file and
deploy its complete stack. Compose builds project Dockerfiles, pulls referenced
images, starts supporting services, preserves named volumes, waits for runtime
readiness, and reconciles the stack on later deployments. The same workflow is
usable across supported application frameworks and custom projects; it is no
longer a Laravel-specific PHP-FPM path.

BonesDeploy still resolves an immutable Git revision, serializes site mutation,
creates and retains filesystem releases, switches `current`, records deployment
state, verifies activation, rolls configuration back to the previous release,
and prunes release directories without deleting Compose volumes. Native
deployments continue using the existing rootless Podman build and native
runtime paths.

Compose-only site diagnostics do not require Podman. Docker Engine and the
Compose plugin are installed from Docker's supported Debian or Ubuntu package
repository so the package source and package names form one coherent contract.

Users receive a clear warning during Docker setup and in documentation that the
Compose file can request authority outside BonesDeploy's native security model.
No application container receives the Docker socket automatically.

Built-in database and cache selection, credentials, provisioning, manifests,
and documentation are absent. Compose users define those services themselves.

## Scope

- Replace the Laravel-specific Docker runtime image, prepare container, command,
  and provisioning behavior with framework-independent Docker Compose behavior.
- Discover the conventional Compose filenames supported by Docker Compose at
  the active project root.
- Delegate Docker image pull and build behavior to Docker Compose from the
  candidate release instead of running the native rootless Podman application
  build pipeline.
- Run the active stack as the stable `bonesdeploy-<site>` Compose project,
  using the protected `shared/.env` file for Compose interpolation and
  preserving Compose named volumes across deploy, rollback, restart, and
  release pruning.
- Keep the existing BonesRemote deployment lock, state, release promotion,
  atomic `current` switch, rollback, and cleanup flow around backend-specific
  Compose operations.
- Provision Docker Engine, Docker Compose, a generic site systemd unit, and
  optional host-nginx ingress for Compose sites through BonesInfra.
- Add Compose-aware preflight, runtime verification, status, doctor, manifest,
  and failure reporting without printing resolved secrets.
- Cover Compose command sequencing, failure propagation, doctor classification,
  manifest inspection failures, and native-to-Compose cleanup with focused
  tests that do not require a Docker daemon.
- Remove built-in database and cache configuration and implementation across
  Rust, Python, generated assets, tests, and documentation.
- Update architecture and security documentation to distinguish native
  guarantees from Compose reduced-guarantee mode.

## Constraints

- Broad compatibility with ordinary Docker Compose stacks takes priority over
  enforcing BonesDeploy's native container policy in Compose mode.
- Docker Compose itself owns Compose parsing, interpolation, builds, service
  dependencies, networks, volumes, and container lifecycle semantics.
- Compose deployments use the conventional rootful Docker daemon. Rootless
  engines are not prerequisites for this change.
- The project Compose file is privileged deployment input. The local project
  operator is trusted to grant the stack the authority it requests.
- BonesDeploy never automatically mounts the Docker socket into a service.
- Compose image builds use `docker compose build`; native deployments retain
  the existing rootless Podman build-script pipeline.
- Compose files and build contexts come from the resolved candidate release,
  never from an unversioned mutable checkout.
- The protected environment is supplied for Compose interpolation. A Compose
  service receives values only when its definition references them through
  `environment` or an `env_file` such as the release's linked `.env`.
- The Compose project name is stable per site. Named volumes and runtime data
  are not stored inside versioned release directories and are never removed by
  release pruning or rollback.
- Rollback restores the previous release's Compose definition and rebuilds or
  reconciles that stack. It does not reverse data mutations in persistent
  volumes or external systems.
- Existing configurations with no backend continue to select native behavior.
- Planning documents require human approval before implementation begins. E2E
  tests are specified but are not run locally by an agent.

## Exclusions

- Rootless Docker, rootless Podman Compose, and per-site Docker daemons.
- A custom Compose parser, Compose-to-`docker run` translation, or a
  BonesDeploy-specific container orchestration format.
- Blocking or rewriting valid Compose features to reproduce native security
  guarantees. This change documents reduced guarantees instead.
- Automatic rollback of database schemas, named-volume contents, or external
  side effects.
- Backing up Docker named volumes through the existing Borg `shared/` backup.
- Registry publishing, registry credential management, multi-host scheduling,
  rolling deployment, blue-green deployment, and zero-downtime guarantees.
- Preserving compatibility with the generated Laravel PHP-FPM Docker runtime;
  Docker projects migrate to a project-owned Compose file.
