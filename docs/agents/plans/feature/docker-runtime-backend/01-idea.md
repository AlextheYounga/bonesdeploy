# Idea

## Request

Add Docker as an optional BonesDeploy application runtime backend without replacing the existing release system. A user selects `native` or `docker` in `[runtime]`; native remains the default. The first Docker implementation supports Laravel, uses rootless Podman for application builds, and uses the conventional rootful Docker daemon only for the running application runtime.

## Problem

BonesDeploy currently runs every application runtime directly on the host. It has no typed runtime-backend selection, no controlled Docker runtime policy, and no way to run a Laravel application's PHP-FPM process in Docker while retaining BonesDeploy's atomic filesystem releases, shared state, host Nginx ingress, and systemd lifecycle. Project-provided Docker Compose would bypass the deployment system's validation and security boundaries.

## Definitions

**Runtime backend:** The selected mechanism that starts the completed application release. `native` starts the framework runtime directly on the host. `docker` starts a BonesDeploy-generated Docker container that bind-mounts the active release. It does not select or alter the application build system.

**Runtime image:** A BonesDeploy-owned Laravel PHP-FPM image containing the operating-system and PHP runtime dependencies. It contains no application source, built assets, dependencies, shared files, or secrets. It is distinct from a release and is rebuilt only when its runtime definition changes.

**Release:** A BonesDeploy filesystem directory under `<project_root>/releases/` produced by the existing build pipeline. A release becomes active through `<project_root>/current`, and is never represented by an application Docker image.

**Prepare container:** A short-lived Docker container that executes existing prepare scripts against a staged release before that release is sealed. It receives read-write mounts for the staged release and the site's shared directory.

**Application container:** The long-lived Docker container started by the site's systemd service. It receives the active release as a read-only mount and the site's shared directory as a read-write mount. It is not privileged and never receives a Docker socket.

**Host-service network:** A BonesDeploy-controlled Docker bridge network that gives Docker-runtime applications a stable host service address. It permits narrowly configured access to BonesDeploy-managed databases and Redis or Valkey without publishing those services externally.

## Desired outcome

A user can initialize a Laravel project with `backend = "native"` or `backend = "docker"`, including through `--runtime-backend`. Existing configuration without `backend` continues to run natively.

A Docker Laravel deployment builds its application release with the existing rootless Podman pipeline, runs prepare scripts in a temporary Docker runtime container, seals and activates the same release filesystem, and restarts the normal site target. The application container mounts `current` read-only and `shared` read-write, serves PHP-FPM to host Nginx through a site-specific Unix socket, and starts again against the previous release after a failed activation.

Docker runtime provisioning installs and operates Docker only through root-owned BonesDeploy infrastructure. Build, deploy, and application users have no Docker daemon access. Doctor and manifest output report Docker runtime health and enforce the runtime mount, identity, privilege, and Docker-socket policy.

## Scope

- Typed `RuntimeBackend` configuration with native defaulting, framework assets, and interactive and non-interactive initialization selection.
- Laravel Docker provisioning: Docker Engine installation, a rootless-Podman-built and root-loaded Laravel runtime image, a site runtime socket directory, host Nginx configuration, and a Docker-backed site systemd service.
- A focused BonesRemote Docker runtime module that constructs validated Docker commands, runs prepare containers, starts and stops application containers, inspects container state, and applies the runtime security policy.
- Docker prepare execution, normal release activation and rollback, Docker-aware manifest and doctor checks, and a controlled host-service network for Laravel access to BonesDeploy-managed databases and Redis or Valkey.
- Focused configuration, provisioning, runtime, diagnostic, native-regression, and Laravel end-to-end coverage, plus documentation of the Docker security tradeoff.

## Constraints

- Application builds remain rootless Podman builds under the existing dedicated build user. That user never receives Docker daemon access.
- Docker runtime mode uses the conventional privileged Docker daemon. Only root-owned BonesDeploy infrastructure controls it. Deploy, build, runtime, and git users never join the `docker` group.
- Docker is a runtime backend, not a second deployment system. The existing release directories, shared directory, `current` symlink, release sealing, activation, rollback, pruning, and site systemd target remain authoritative.
- BonesDeploy constructs Docker argument vectors with validated site configuration through `std::process::Command`. It does not execute project Compose files or accept arbitrary Docker flags, mounts, devices, capabilities, security options, or networks from `bones.toml`.
- Nginx, TLS, firewall, fail2ban, BonesRemote, Git repositories, and BonesDeploy-managed databases and Redis or Valkey remain on the host.
- The Laravel application container runs as the site's runtime identity where Docker supports the host UID and GID mapping, drops capabilities, enables no-new-privileges, uses no privileged mode, mounts no Docker socket or host root filesystem, exposes the release read-only, and limits writable mounts to declared shared and runtime socket paths.
- Host Nginx reaches Docker Laravel PHP-FPM over a site-specific Unix socket. Docker application services do not use host networking or publish public application ports.
- This work follows the existing Acta planning gate. Implementation starts only after a human approves these planning documents. E2E tests are not run locally by an agent.

## Exclusions

- Docker support for Django, Rails, Next, Nuxt, SvelteKit, and static frameworks.
- Arbitrary Docker Compose execution, project-specified Docker command-line flags, application images, registry publishing, and application-image rollback.
- Rootless Docker runtime support, Docker daemon access for non-root accounts, host networking, public database listeners, and unrestricted inter-site Docker networking.
- Replacing host Nginx, TLS, databases, Redis or Valkey, Git repositories, or BonesRemote with containers.
- Rewriting existing Laravel prepare scripts or user-managed `.env` files.
