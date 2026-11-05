# Tasks

## Implementation

- [x] Add `RuntimeBackend` to Rust Core with lowercase TOML serde, native defaulting, configuration validation, framework defaults, and tests proving omitted and explicit native configuration retain current behavior.
- [x] Add `--runtime-backend` and the Native/Docker initialization choice, materialize it in generated `bones.toml`, and cover interactive and non-interactive initialization behavior.
- [x] Add a focused BonesInfra Docker Engine service that installs, enables, and starts Docker only for Docker-backend sites without granting daemon access to deploy, build, runtime, or git users.
- [x] Add BonesDeploy-owned Laravel runtime image assets and provisioning that builds the runtime image through the dedicated build identity with rootless Podman, saves a Docker archive, loads it as root, and creates a content-derived tag plus the stable runtime tag.
- [x] Add backend-aware Laravel provisioning: preserve native host PHP-FPM provisioning; for Docker, create the runtime socket directory, render Docker PHP-FPM socket configuration and host Nginx configuration, and register the Docker application service in the site target.
- [x] Add the BonesRemote runtime boundary and root-only runtime commands, with validated container and image names, Docker argument-vector construction, start, stop, and explicit application security policy.
- [x] Route Docker backend prepare execution through temporary containers with read-write staged-release and shared mounts while preserving existing script ordering, prelude streaming, working directory, and conceptual environment; retain host-user prepare execution for native sites.
- [x] Connect the Docker application systemd unit to BonesRemote start and stop operations so target restarts run the active `current` release read-only, mount shared state and the PHP-FPM socket directory with the required write access, and preserve existing activation rollback behavior.
- [ ] Provision the controlled Docker bridge network and stable host service name, constrain BonesDeploy-managed database and Redis or Valkey reachability to that network, and retain host-side authentication and non-public listeners.
- [ ] Extend BonesInfra manifest declarations and BonesRemote doctor checks with the complete Docker backend, engine, image, container, service, socket, mount, identity, Docker-socket, and privileged-container evidence.
- [ ] Update Laravel Docker fixtures and ignored E2E coverage for provisioning, SQLite deployment, socket ingress, shared persistence, second deployment, failed activation rollback, container security, and managed-service network connectivity.
- [x] Update README and security invariants to describe optional Docker runtime mode and its rootful-daemon security tradeoff without changing native-mode claims.

## Validation

- [x] Run focused Rust Core, local CLI, BonesRemote, and BonesInfra checks that cover backend selection, provisioning parsing, Docker command policy, prepare behavior, and native regressions; exclude `e2e` execution.
- [x] Run `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff check .`, and `ruff format .`, then resolve every reported warning or error.
- [ ] Review ignored Laravel Docker E2E test definitions to confirm they assert the full provisioning, deployment, rollback, filesystem, socket, security, and managed-network outcomes.

## Completion

- [ ] Review the final diff to confirm native runtime paths do not invoke Docker and no implementation grants Docker group membership, exposes the Docker socket, accepts arbitrary Docker instructions, uses privileged containers, host networking, public application ports, or writable release mounts.
- [x] Record validation results, material deviations, and deliberately unfinished work below.

## Completion notes

The implementation currently delivers configuration, init selection, Laravel Docker provisioning primitives, runtime image archive handoff, Docker prepare execution, root-only runtime start/stop, restricted mounts, and basic manifest/doctor awareness. Managed host-service networking, full container inspection, complete security evidence, and Laravel Docker E2E coverage remain unfinished and are intentionally not marked complete.

Focused Rust tests, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt`, `shfmt -w .`, Python compilation, `ruff check .`, and `ruff format .` passed. The normal `bonesdeploy` doctor integration test was not included in the passing run because its temporary Git repository attempted a GPG-signed commit and the environment cancelled pinentry; this is an environment failure, not a feature assertion failure. The repository's E2E tests were not run as required.

After merging `refactor/project-infra` from develop, the Docker implementation was re-homed into the project-owned infrastructure package: the Laravel Docker provisioning module and its `Containerfile`, `www.conf`, and systemd unit templates now ship as `.bones/infra/docker.py` and `.bones/infra/templates/docker/` under `crates/bonesdeploy/assets/frameworks/laravel/infra/`, with the Laravel `runtime.py` and `manifest.py` dispatching on the selected backend. The BonesInfra Engine service module, its Python-package assets, and the core manifest Docker branches were removed in favor of the project manifest API. All 238 Python tests and the full Rust non-E2E suites pass on the merged tree.
