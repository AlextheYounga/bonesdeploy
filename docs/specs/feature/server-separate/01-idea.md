# Idea

## Request

Separate provisioning into host-wide `server` commands and per-project `site`
commands. `server setup` establishes a reusable host baseline. `site setup`
registers and provisions exactly one project only after that baseline is
present. Keep root `bonesdeploy setup` as an idempotent convenience workflow
that runs `server setup` followed by `site setup`. Keep root `bonesdeploy
doctor` as a composed check that runs both `server doctor` and `site doctor`.
Replace the flat inspection and `remote` command surface with `server` and
`site` command groups.

## Problem

The current `bonesinfra setup apply` combines host policy with project state,
and `bonesdeploy setup` invokes that combined operation before services,
runtime, and doctor. A project configuration therefore controls shared host
mutations such as packages, firewall rules, the global deploy user, the
shared Podman image store, and global sudoers. The flat CLI and combined skill
readiness check also cannot distinguish a missing host baseline from a missing
site. This makes a second project on an existing host ambiguous and permits
site setup to leave partial state when the host has not been provisioned.

## Definitions

**Server baseline:** The host-wide, idempotent provisioning state shared by all
BonesDeploy projects on one Debian or Ubuntu server. It includes packages,
hardening, firewall, fail2ban, unattended upgrades, the shared Podman image
store and base image, the global `git` deploy identity and authorized keys,
BonesRemote state roots and binary, and the global BonesDeploy sudoers policy.
It contains no project, runtime, service, DNS, framework, or release state.

**Site:** One BonesDeploy project identified by its project name and isolated
per-project identities, repository, paths, control-plane state, runtime,
services, and releases. A site shares a server baseline but does not own or
modify it.

**Server readiness:** The successful host-mode result of `bonesremote doctor`
run by `bonesdeploy server doctor`. It proves the baseline artifacts required
before a site may be provisioned.

**Site base provisioning:** The non-framework portion of one site's setup:
its runtime and build identities, isolated build state, bare repository, site
directories, root-owned BonesRemote site configuration, and placeholder
release/current link. It does not install selected services, configure a
framework runtime, obtain TLS certificates, publish secrets, push Git data, or
deploy a release.

## Desired outcome

A fresh host follows `bonesdeploy init`, `bonesdeploy server setup --yes`, then
`bonesdeploy site setup --yes`. A second project targeting the same ready host
follows `bonesdeploy init` then `bonesdeploy site setup --yes` without repeating
host provisioning. `site setup` first verifies server readiness; when the
baseline is absent it creates no site state and prints `Next: bonesdeploy
server setup --yes`.

Server setup and server doctor work from only the SSH host, user, and port.
Site setup executes exactly: server readiness check, site base provisioning,
site services, site runtime, and site doctor. Re-running either setup preserves
the established baseline or site and completes safely. Public help, embedded
guidance, and tests expose only the new command hierarchy.

## Scope

- Split the Python provisioning context, orchestrator, operations, and CLI
  entry points along server and site ownership boundaries.
- Establish and inspect the complete server baseline, including global
  BonesRemote roots that exist before any site.
- Build `bonesdeploy server` and `bonesdeploy site` command groups and move
  project inspection commands under `site`.
- Guard site setup with server readiness before any site mutation.
- Split skill readiness and next-step guidance into server and site states.
- Update focused tests, the shared-server E2E harness, user guidance, embedded
  skill documents, architecture and security documentation, and `CONTEXT.md`.

## Constraints

- Support only Debian and Ubuntu hosts.
- `ServerContext` contains only `HOST`, `SSH_USER`, and `PORT`; server
  provisioning must not receive or inspect site configuration.
- The canonical project `.env` remains the source of SSH connection values.
  No server registry, `server.toml`, or other local server configuration file
  is introduced.
- `site setup` never pushes Git or secrets, configures SSL, or deploys.
- Server setup is global and site setup is per-project; both are idempotent.
- Remove old command forms rather than retaining aliases, including the flat
  `status`, `manifest`, and `releases` commands, the full `remote` namespace,
  and hidden `guide`. Keep root `setup` and `doctor` as the documented composed
  workflows described above.
- Do not run E2E tests during implementation; leave the updated suite ready for
  the repository owner.

## Exclusions

- No change to the deployment, rollback, secrets, init, skill, update, or
  version top-level command responsibilities.
- No server inventory, host registration workflow, multi-host configuration,
  or persisted server metadata.
- No automatic server provisioning from a site command.
- No change to the project-specific runtime, service, SSL, manifest, patch, or
  release behavior beyond relocating its command surface and enforcing the new
  provisioning boundary.
