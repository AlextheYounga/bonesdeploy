# Plan

## Current behavior

`crates/bonesdeploy-core/src/config.rs` owns the typed `Runtime` TOML model. It has no backend field; its `Runtime::default` supplies template, web-root, node-version, shared-path, and permission defaults. Rust Core and BonesInfra both parse the same `bones.toml` configuration, while BonesInfra retains unrecognized runtime values in its runtime data map.

`bonesdeploy init` accepts framework selection and framework variables in `crates/bonesdeploy/src/cli/args.rs`; `commands/init/config.rs`, `commands/init/framework.rs`, and `ui/prompts.rs` collect them and materialize the selected framework assets. It has no runtime backend prompt or flag.

`crates/bonesremote/src/commands/deploy/lifecycle.rs` already runs a staged lifecycle: export source, rootless Podman build, promote, wire shared symlinks, prepare, seal, preflight, atomically activate `current`, restart the site's systemd target, and restore the previous `current` target plus restart on failed activation. `release/lifecycle/prepare.rs` executes numbered prepare scripts as the runtime user on the host. Build containers are rootless Podman containers started through the dedicated build user's systemd manager; they are the repository's only current container runtime.

`crates/bonesinfra/python/src/bonesinfra/frameworks/base.py` provisions Laravel through `PHPFramework`: it installs host PHP and PHP-FPM, renders a PHP-FPM pool and host Nginx FastCGI configuration, and does not create a Laravel application systemd service. Existing systemd helpers register service requirements in the site target. The existing host Nginx runtime socket and PHP-FPM socket paths are derived in BonesInfra path and language services.

`crates/bonesinfra/python/src/bonesinfra/cli/commands/setup/image_store.py` provisions a rootless Podman image store for the build pipeline. `setup/users.py` creates dedicated build and runtime identities, and the sudoers template grants the deploy user only narrow `bonesremote` hook, restart, and release operations. Docker is not installed, no non-root account has daemon access, and no Docker runtime image state exists.

`crates/bonesinfra/python/src/bonesinfra/manifest.py` inventories managed host artifacts and services. `crates/bonesremote/src/commands/doctor/` performs root-owned site and security checks, including release sealing and Podman readiness. Neither diagnoses Docker daemon, runtime image, container, mounts, or container security state. The Laravel E2E harness is in `e2e/` and currently asserts host PHP-FPM behavior.

## Intended behavior

`RuntimeBackend::Native` is the serde default, so configuration without `runtime.backend` retains the exact native provisioning, prepare, service, and deployment behavior. Initialization writes the selected lowercase backend and accepts the same selection through `--runtime-backend`.

For a Laravel Docker site, BonesInfra installs and validates Docker Engine, provisions the Docker PHP-FPM socket directory, renders host Nginx for that socket, builds a BonesDeploy-owned runtime image with rootless Podman, saves it as a Docker archive, and loads it through root-owned Docker. Its generated Docker application systemd unit joins the existing site target and calls root-only `bonesremote runtime start --site <site>` and `bonesremote runtime stop --site <site>`.

Deploy continues to create an ordinary release, wire the existing shared symlinks, seal the release, atomically repoint `current`, restart the site target, and restore `current` after a restart failure. Backend selection changes only prepare and the application process. Docker prepare uses a temporary container with read-write staged-release and shared mounts. The application container uses the loaded runtime image, mounts `current` at `/app` read-only, mounts the project shared directory at its unchanged absolute host path read-write, and exposes PHP-FPM only through the site socket mounted from `/run/bonesdeploy/<site>/php`.

Docker-runtime Laravel applications reach configured host-managed services on a BonesDeploy-controlled bridge network at `host.bonesdeploy.internal`. Host services remain non-public and application configuration remains user-managed. Manifest and doctor report Docker backend state and fail for missing Docker prerequisites, incorrect mount policy, or an unsafe running container.

## Approach

Add `RuntimeBackend` beside the existing typed core runtime configuration, default it to native with serde, pass it through init scaffolding, and add focused config and CLI tests. Do not add a general Docker configuration section; Laravel derives its runtime image, identity, paths, socket, and network from configuration that already exists.

Branch explicitly at the existing framework-provisioning and prepare-execution boundaries. Native Laravel continues to use the current PHP and PHP-FPM provisioning path. Docker Laravel uses a focused Docker service for engine installation and a Laravel Docker provisioner for the runtime image, socket directory, Nginx FastCGI configuration, service unit, and target registration. This keeps common project, user, release, Nginx, TLS, firewall, and database provisioning intact.

Create a `bonesremote::runtime` boundary with native and Docker implementations. The Docker implementation owns validated container and image names, runtime-image archive loading, argument-vector command construction, separate prepare and application mount policies, container lifecycle operations, and inspection. The privileged runtime CLI only loads registered site configuration and performs fixed operations; it accepts no project-provided command or Docker arguments.

Build the Laravel runtime image during runtime provisioning under the existing rootless Podman build identity, hash the Containerfile and PHP runtime definition into a `bonesdeploy/laravel-<site>:<runtime-hash>` tag, save a `docker-archive`, and let the root-owned provisioning process load that archive into Docker. Ordinary deployments never rebuild the runtime image. The systemd wrapper always starts the exact provisioned tag.

Run the existing ordered prepare scripts by streaming the existing BonesRemote function prelude and script content into a temporary Docker container whose working directory is the staged release. Preserve the existing conceptual prepare environment. Keep Docker prepare mounts explicit and writable; keep running application mounts separately explicit and release-read-only. Seal the prepared release through the existing lifecycle before activation.

Add a site-specific Docker application service to the existing target, leaving `bonesremote service restart` as the deployment and rollback orchestration point. On each restart, the wrapper removes any stale named application container and runs the fixed command against `current`; the existing activation failure path then restores the old symlink and restarts that same service.

Extend existing manifest declarations and remote doctor checks with Docker artifacts, daemon and image health, service and container state, Unix-socket presence, release and shared mount sources and modes, expected container identity, Docker-socket absence, and privileged-mode absence. Provision a managed bridge network plus constrained host-service reachability and use a stable internal host name; retain host database authentication and non-public listeners.

Update security and user documentation to state the rootful Docker daemon tradeoff, rootless Podman build separation, controlled runtime definitions, host-owned ingress, and runtime mount policy. Adapt the Laravel E2E fixture and assertions to exercise the Docker lifecycle with SQLite, shared persistence, activation rollback, and container security, then add managed-service network coverage.

## Responsibilities and boundaries

`bonesdeploy-core` owns the typed `RuntimeBackend`, its default, TOML serialization, and validation. The local `bonesdeploy` init command owns user selection, argument parsing, and materializing the selected backend into project configuration.

BonesInfra owns host provisioning. Its Docker service installs, enables, starts, and checks Docker Engine; its Laravel Docker path provisions the runtime image, archive handoff, Docker load, socket directory, Nginx configuration, systemd unit, target membership, bridge network, and managed-service listener rules. The native Laravel path remains the owner of host PHP and PHP-FPM provisioning.

BonesRemote owns privileged deployment-time runtime operations. Its runtime module translates validated site configuration into Docker commands, runs prepare containers, starts and stops application containers, and returns inspection evidence. The deployment lifecycle remains the owner of release staging, shared wiring, sealing, activation, rollback, and restart orchestration; it delegates only backend-specific prepare execution.

The existing systemd target remains the site-level lifecycle boundary. The Docker application systemd unit owns start and stop delegation to BonesRemote, while `bonesremote service` continues to restart and verify the target as a whole.

BonesInfra manifest declarations own provisioned Docker artifacts and services. BonesRemote doctor owns privileged runtime inspection and enforcement of container security invariants. Documentation owns the user-facing security model and operational limits.

## Affected areas

- `crates/bonesdeploy-core/src/config.rs`, configuration tests, and framework `bones.toml` assets for backend representation and native-default coverage.
- `crates/bonesdeploy/src/cli/args.rs`, `commands/init/`, `ui/prompts.rs`, init tests, and framework asset selection for `--runtime-backend` and the interactive choice.
- `crates/bonesremote/src/release/lifecycle/prepare.rs`, deployment lifecycle dispatch, command arguments and dispatch, and a new focused `crates/bonesremote/src/runtime/` module for Docker command, prepare, service, and inspection behavior.
- `crates/bonesremote/src/commands/doctor/` and associated tests for Docker runtime diagnostics and security checks.
- `crates/bonesinfra/python/src/bonesinfra/config/`, `services/`, `frameworks/base.py`, `frameworks/laravel.py`, systemd and Nginx assets, manifest declarations, CLI runtime provisioning, and Python tests for backend-aware Laravel provisioning.
- New BonesDeploy-owned Laravel runtime Containerfile and PHP-FPM configuration assets embedded with BonesInfra.
- `e2e/` Laravel fixtures and tests for Docker runtime lifecycle behavior; these tests remain ignored and are not executed locally during this work.
- `README.md`, `docs/architecture/security-model.md`, `docs/security/invariants.md`, and applicable context documentation for the Docker runtime model.

## Decisions

- Native is the `RuntimeBackend` default so existing configurations and provisioning behavior remain unchanged without migration.
- The first Docker backend supports Laravel only. Laravel establishes the complete PHP-FPM, shared filesystem, socket ingress, prepare, activation, and rollback mechanics before another framework is added.
- Runtime images contain only runtime dependencies, never an application release. Filesystem releases remain the rollback unit, which preserves atomic `current` restoration without image rebuilding or retagging.
- Rootless Podman builds the runtime image and emits a Docker archive; root loads it into the Docker daemon. This maintains build-user isolation from the privileged daemon.
- Docker commands are built by BonesRemote as argument vectors from validated configuration. This prevents Compose files and arbitrary flags from becoming privileged Docker instructions.
- Prepare and application containers use distinct mount definitions because their authorities differ. Prepare writes the staged release before sealing; the application reads the active sealed release and writes only shared state and its socket directory.
- Docker application control remains behind a BonesRemote systemd wrapper instead of an expanded systemd `docker run` command. Validation and security policy stay in the privileged Rust boundary.
- Host Nginx connects over a site-specific Unix socket. Docker neither publishes PHP-FPM nor uses host networking.
- A BonesDeploy-controlled bridge network and stable host name provide managed-service connectivity. User `.env` content is not rewritten, and host services stay non-public.
- Docker changes the runtime security model because its daemon is privileged. Docker engine control is root-only, applications receive no engine socket, and documentation states that native and Docker modes do not have identical security properties.

## Risks

- Backend-default deserialization or init-scaffolding errors can change existing native sites. Config, init, and native provisioning regression coverage must prove that an omitted backend selects native and does not require Docker.
- A Docker command, mount source, container name, or image tag built from unvalidated input can become a privileged daemon-control path. Runtime construction must use validated site values and typed argument arrays.
- A missing image, stopped daemon, absent socket directory, or failed PHP-FPM startup can fail after activation. The existing rollback path must restore `current` and restart the Docker service successfully.
- Incorrect mount modes or identity mapping can make releases writable, prevent shared persistence, or produce unreadable files. Runtime inspection and lifecycle tests must verify release read-only behavior, declared shared writes, and expected ownership.
- Docker bridge connectivity can accidentally broaden database exposure. Host service listener and firewall configuration must restrict reachability to the managed Docker network while retaining database-scoped credentials.
- Existing documentation states that container execution is rootless. The security documentation must be revised precisely so the rootless-build guarantee is not incorrectly applied to the rootful Docker runtime.

## Validation

- Rust configuration and init tests prove omitted and explicit `native` backends retain native behavior, `docker` serializes as lowercase TOML, and interactive and non-interactive initialization record the selected backend.
- Python provisioning tests prove Docker Engine is selected only for Docker sites, native Laravel continues to provision host PHP and PHP-FPM, Docker Laravel provisions the runtime image, socket, Nginx, systemd target requirement, and no Docker daemon access for non-root identities.
- BonesRemote unit and integration tests prove Docker command construction rejects invalid site-derived values, uses no shell concatenation, distinguishes prepare and application mounts, preserves prepare environment and script order, and enforces no privileged mode or Docker socket mount.
- Lifecycle tests prove a Docker prepare container can modify a staged release and shared state before sealing, the application container cannot modify the sealed active release, shared state persists across restart and deployment, and failed Docker service startup restores and serves the previous release.
- Manifest and doctor tests report backend, daemon, image, service, container, socket, mount, identity, Docker-socket, and privileged-container state; Docker prerequisites and unsafe runtime state fail clearly.
- Ignored Laravel Docker E2E coverage proves provisioning, first deployment, Nginx-to-PHP-FPM socket traffic, second deployment, rollback after failed activation, security checks, SQLite operation, and managed database or Redis/Valkey connectivity. It is reviewed but not run locally by an agent.
- Run affected Rust and Python test suites excluding `e2e`, `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff check .`, and `ruff format .`. Review the final diff for Docker group grants, Docker socket exposure, privileged containers, arbitrary daemon instructions, accidental host-network use, and unintended native-path changes.
