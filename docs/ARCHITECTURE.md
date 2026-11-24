# Architecture

The exhaustive module inventory lives in
[`docs/architecture/reference.md`](architecture/reference.md). This document
answers the more important question: where does behavior belong?

## System shape

```text
bonesdeploy-core                 bonesdeploy
  Bones configuration             local CLI and orchestration
  DeploymentPaths                 prompts, Git, GPG, SSH
  shared validation                      │
          ▲                              ▼ direct Rust calls
          │                          bonesinfra
          │                            provisioning plans
          │                            RustInfra SSH execution
          │                            framework and Linux services
          │                              │
          └──────────────────────────────┤ shared types
                                         ▼
                                    target host

bonesdeploy ──SSH exact commands──> bonesremote
                                    release lifecycle, locking,
                                    activation, rollback, shared data
```

- `bonesdeploy-core` owns the vocabulary shared by every Rust component:
  `Bones`, validation, remote deployment descriptors, and `DeploymentPaths`.
- `bonesdeploy` owns user interaction and orchestration. Provisioning command
  handlers are thin: load `Bones`, call a typed BonesInfra entry point, and
  present the result.
- `bonesinfra` is a Rust library. It derives a `ProvisioningContext` directly
  from `Bones`, prepares RustInfra operations, and executes them over SSH. It
  has no subprocess, JSON request, Python, wheel, pip, or virtualenv boundary.
- `bonesremote` owns deployment-time state changes. It never calls BonesInfra.

## Dependency direction

`bonesdeploy-core` is the shared leaf. `bonesinfra`, `bonesdeploy`, and
`bonesremote` depend on it. `bonesdeploy` depends on `bonesinfra` and invokes it
directly. BonesInfra must not depend on CLI presentation or BonesRemote release
internals.

RustInfra is the generic provisioning engine. Generic facts and operations
belong there. BonesInfra operations should represent BonesDeploy domain
behavior such as installing a backup passphrase or resolving a site manifest.

## Ownership map

| Responsibility | Canonical owner |
| --- | --- |
| Project configuration and validation | `bonesdeploy-core::config` |
| Product-owned filesystem layout | `bonesdeploy-core::paths::DeploymentPaths` |
| CLI prompts and command composition | `crates/bonesdeploy/src/commands` |
| SSH/Git/GPG integrations | `crates/bonesdeploy/src/infra` and `commands/secrets` |
| Provisioning inventory and execution | `crates/bonesinfra/src/execution.rs` |
| Built-in framework runtime selection | `crates/bonesinfra/src/frameworks/` |
| Server/site/runtime/SSL/tunnel plans | `crates/bonesinfra/src/provisioning/` |
| Manifest declarations, inspection, deletion safety | `crates/bonesinfra/src/manifest/` |
| Version-gated infrastructure migrations | `crates/bonesinfra/src/patches.rs` |
| Release lifecycle and site mutation locks | `crates/bonesremote/src/release` |
| Managed template sources | `crates/bonesinfra/templates/` |
| Project-authoritative template snapshot | `infra/templates/` |

## Project-owned infrastructure

Initialized projects contain:

```text
infra/
├── templates/   # managed, committed, inspectable rendering inputs
└── secrets/     # encrypted project secrets
```

BonesDeploy embeds the source templates from `crates/bonesinfra/templates/`,
atomically refreshes `infra/templates/`, and prunes unselected framework
directories. Provisioning always resolves templates from the project snapshot.

`infra/custom/` is temporarily unsupported. New projects do not create it. Any
command that could provision or refresh infrastructure rejects an existing
directory before changing managed files, leaving the project-owned content
untouched for deliberate archival or removal.

## Frameworks

Each built-in framework module under `crates/bonesinfra/src/frameworks/` owns
its validated runtime plan: mode, runtime service, language, placeholder,
network shape, writable paths, and project-local template names. Shared Linux
behavior stays in provisioning modules rather than being copied into each
framework.

Native frameworks use host systemd, nginx, and AppArmor. Docker-backed sites
install Docker Engine and the Compose plugin, run the project stack through a
generic systemd unit, and add managed nginx ingress only when a Compose port is
configured. BonesRemote remains responsible for release-time Compose commands.

## Provisioning flow

1. A command loads and validates `Bones`.
2. BonesInfra derives `ProvisioningContext` and canonical `DeploymentPaths`.
3. `TemplateResolver` resolves only committed `infra/templates/` files.
4. A focused provisioning module builds a typed `PreparedDeploy`.
5. `ProvisioningRuntime` executes the plan through RustInfra and returns stable
   operation accounting.

Server setup also validates Debian 12+ or Ubuntu 24.04+ on x86_64 before
mutation. Trust-boundary values such as project names, hosts, ports, domains,
emails, framework booleans, versions, and internal ports are validated before
they enter paths or commands.

## Manifest and deletion

The typed manifest combines common, service, backend, and framework
declarations. Read-only RustInfra facts resolve artifact kind/state and systemd
health. JSON output contains metadata only, never file contents or secrets.

Deletion resolves a secret-free plan locally, displays it for confirmation,
serializes it, and re-parses/revalidates it immediately before execution.
Validation permits only site-owned or precisely derived system paths and
services. The Borg repository and passphrase are retained; the scheduled cron
entry is removed. Services stop before artifacts are removed, and nested paths
are removed before their parents.

## Adding behavior

- Add a global setting or validation rule in `bonesdeploy-core`.
- Add a product-owned path to `DeploymentPaths`.
- Add a framework runtime in `bonesinfra/src/frameworks/` and its templates in
  `bonesinfra/templates/frameworks/<name>/`.
- Add shared host behavior to the focused BonesInfra provisioning module.
- Add a generic reusable fact or operation to RustInfra rather than wrapping a
  private command abstraction.
- Add deployment-time state changes to the existing BonesRemote lifecycle and
  acquire its site mutation lock.

Do not add a second configuration model, provisioning transport, release
lifecycle, path vocabulary, or SSH implementation.
