# Server And Site Command Split

<!-- Merged from feature/server-separate -->

## Idea

### Request

Separate provisioning into host-wide `server` commands and per-project `site`
commands. `server setup` establishes a reusable host baseline. `site setup`
registers and provisions exactly one project only after that baseline is
present. Keep root `bonesdeploy setup` as an idempotent convenience workflow
that runs `server setup` followed by `site setup`. Keep root `bonesdeploy
doctor` as a composed check that runs both `server doctor` and `site doctor`.
Replace the flat inspection and `remote` command surface with `server` and
`site` command groups.

### Problem

The current `bonesinfra setup apply` combines host policy with project state,
and `bonesdeploy setup` invokes that combined operation before services,
runtime, and doctor. A project configuration therefore controls shared host
mutations such as packages, firewall rules, the global deploy user, the
shared Podman image store, and global sudoers. The flat CLI and combined skill
readiness check also cannot distinguish a missing host baseline from a missing
site. This makes a second project on an existing host ambiguous and permits
site setup to leave partial state when the host has not been provisioned.

### Definitions

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

### Desired outcome

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

### Scope

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

### Constraints

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

### Exclusions

- No change to the deployment, rollback, secrets, init, skill, update, or
  version top-level command responsibilities.
- No server inventory, host registration workflow, multi-host configuration,
  or persisted server metadata.
- No automatic server provisioning from a site command.
- No change to the project-specific runtime, service, SSL, manifest, patch, or
  release behavior beyond relocating its command surface and enforcing the new
  provisioning boundary.


## Plan

### Current behavior

`crates/bonesinfra/python/src/bonesinfra/cli/commands/setup/__init__.py`
defines `deploy_setup(ctx, bonesremote_version)`. Its single sequence installs
base and supplementary packages; applies `algif_aead` hardening; establishes
and seeds the image store; creates deploy, runtime, and build identities;
creates site paths and a placeholder; configures firewall, fail2ban, and
unattended upgrades; installs authorized keys and `bonesremote`; then writes
sudoers. The `setup` package owns both host-wide and site-specific operations.

`DeployContext` in `bonesinfra/config/context.py` embeds `ServerConfig` inside
`AppConfig` alongside project, DNS, deploy, runtime, and service settings.
`pyinfra/runner.py` accepts `DeployContext` and obtains connection details via
`ctx.app.server`, so every provisioning invocation receives the entire project
configuration.

The embedded Python CLI in `bonesinfra/cli/app.py` exposes `setup apply`,
`runtime apply`, `services apply`, `ssl apply`, and `helpers apply`.
`bonesdeploy` exposes flat `Setup`, `Doctor`, `Status`, `Manifest`, and
`Releases` variants plus `RemoteCommand` in `src/cli/args.rs`. Its root setup
orchestrator in `commands/setup.rs` runs remote bootstrap, services, runtime,
and site doctor. `commands/skill.rs` uses one `remote_setup_complete` probe and
cannot report server and site readiness independently.

`bonesremote doctor` already supports a host invocation and an optional
`--site` invocation, but its host checks cover only a subset of the intended
baseline. The current E2E `SampleProject` setup flow runs `bonesdeploy setup
--yes` for every framework project, including projects that share an Incus
server.

### Intended behavior

`bonesinfra server apply --env-file .env --bonesremote-version <version>`
parses only `ServerContext` and provisions the server baseline. `bonesinfra
site apply --env-file .env` parses `DeployContext` and performs only site base
provisioning. The runtime, services, SSL, manifest, helpers, and patches remain
independent internal operations.

`bonesdeploy server setup --yes` applies the baseline and runs server doctor.
`bonesdeploy server helpers --yes` installs optional host-wide tools.
`bonesdeploy server doctor` connects with the privileged SSH identity and runs
`bonesremote doctor` without `--site`.

Root `bonesdeploy setup --yes` remains a convenience orchestrator. It invokes
`bonesdeploy server setup --yes` first and then `bonesdeploy site setup --yes`.
It does not contain provisioning operations of its own and stops when either
delegated setup fails. Because both delegated setup paths are idempotent,
repeating root setup is safe for an existing host and site.

Root `bonesdeploy doctor` remains a convenience diagnostic. It runs
`bonesdeploy server doctor` and `bonesdeploy site doctor`, reports both results,
and returns failure when either check fails. The composed doctor invokes both
checks even when the first reports a failure so one command exposes the complete
host and site diagnosis. `bonesdeploy server doctor` and `bonesdeploy site
doctor` remain independently callable; `site doctor --local` continues to skip
remote checks.

`bonesdeploy site setup --yes` first runs the same server readiness check. On
failure it stops before `site apply` and reports the exact server-setup next
command. On success it calls site base provisioning, site services, site
runtime, and site doctor in that order. Site runtime, services, and SSL remain
available as independent `site` subcommands. Site doctor invokes `bonesremote
doctor --site <project>`; site status, manifest, and releases move under the
same group. `rollback` remains top-level.

The host doctor verifies the supported OS, Podman, AppArmor, global deploy
identity, installed BonesRemote binary, root-owned global state directories,
valid global sudoers policy, shared image-store configuration and seeded image,
and expected host security services. The skill state sequence is
uninitialized, server missing, site missing, SSL missing, and ready. It directs
each project at a ready shared host to site setup.

### Approach

Introduce `ServerContext` in the existing Python context module, parsing only
`HOST`, `SSH_USER`, and `PORT` from the supplied `.env`. Reshape
`DeployContext` to hold a server connection plus site configuration rather than
placing the connection inside `AppConfig`. Give both contexts the shared server
connection interface required by the pyinfra runner; change the runner's type
and field access to that interface. This construction prevents server
operations from reaching project, runtime, service, DNS, or framework values.

Replace the `setup` command package with `server` and `site` packages. Move
host operations into server modules: packages, deploy user, BonesRemote,
global state roots, and sudoers. Move site operations into site modules:
identities, directories, and placeholder. Move the shared image-store contract
from the command package to `services/linux/image_store.py`, so site build-user
validation depends on neutral Linux service behavior rather than a server
command module. Define explicit linear `deploy_server_setup()` and
`deploy_site_setup()` orchestrators and test their call boundaries.

Add Python `server apply` and `site apply` commands, retaining the current
separate operations under their existing command names. Extend BonesRemote
host doctor and its security checks to model the global roots and baseline
artifacts created by server provisioning. Server setup creates global
BonesRemote roots before any site exists.

Replace the flat Rust variants and `RemoteCommand` with `Command::Server` and
`Command::Site` subcommands, while retaining root `Command::Setup` and
`Command::Doctor` as thin composition commands. Create focused `commands/server/` and
`commands/site/` modules, update dispatch and command tests, and delete the
obsolete flat command modules and hidden guide path. The root setup command
delegates to server setup and then site setup; the root doctor command delegates
to server doctor and then site doctor, aggregating their results without owning
check details. The site setup command owns the fixed readiness/base/services/
runtime/doctor sequence, while server setup owns baseline apply/doctor. Split
skill probing into server readiness and site readiness functions and update all
command strings it emits.

Update the E2E shared-server lifecycle so the Incus server receives one server
setup before framework projects execute site setup. Update all living
documentation and embedded skill assets to use the new command surface and the
exact site setup sequence. Historical planning documents remain unchanged.

### Responsibilities and boundaries

| Boundary | Responsibility |
| --- | --- |
| `bonesinfra.config.context.ServerContext` | Validated SSH connection data for server provisioning only. |
| `bonesinfra.config.context.DeployContext` | One site's server connection and project, runtime, service, DNS, deploy, and path configuration. |
| `bonesinfra.pyinfra.runner` | Connect a context that exposes the shared server connection; it owns no provisioning policy. |
| `bonesinfra.cli.commands.server` | Idempotent server baseline orchestration and host-wide operations. |
| `bonesinfra.cli.commands.site` | Idempotent site base orchestration and project-specific identities, directories, and placeholder release. |
| `bonesinfra.services.linux.image_store` | Shared image-store configuration and validation used by the relevant host and build-user operations. |
| `bonesinfra` runtime, services, SSL, manifest, helpers, and patches commands | Their existing distinct provisioning or inspection responsibilities. |
| `bonesremote doctor` host mode | Read-only verification of global server readiness. |
| `bonesremote doctor --site` | Read-only verification of one imported site's identity, paths, repository, and configured runtime state. |
| `bonesdeploy commands/server` | Server CLI confirmation, baseline apply, and host doctor orchestration. |
| `bonesdeploy commands/site` | Site CLI confirmation, readiness guard, site provisioning sequence, and project-scoped inspection/actions. |
| `bonesdeploy commands/skill` | State-aware guidance assembled from independent server, site, and SSL readiness results. |
| E2E harness | Provision the shared host once, then exercise isolated site setup per sample project. |

### Affected areas

- `crates/bonesinfra/python/src/bonesinfra/config/context.py` and
  `pyinfra/runner.py`.
- `crates/bonesinfra/python/src/bonesinfra/cli/app.py` and the replacement
  `cli/commands/server/` and `cli/commands/site/` packages.
- Current setup operations under
  `crates/bonesinfra/python/src/bonesinfra/cli/commands/setup/`, including
  `users.py`, `directories.py`, `placeholder.py`, `image_store.py`,
  `bonesremote.py`, and `sudoers.py`.
- `crates/bonesinfra/python/src/bonesinfra/services/linux/image_store.py` and
  imports from build-user validation.
- `crates/bonesremote/src/commands/doctor/` and associated security/path
  checks for host-baseline evidence.
- `crates/bonesdeploy/src/cli/args.rs`, `cli/dispatch.rs`, `commands/mod.rs`,
  new `commands/server/` and `commands/site/` modules, and removed flat or
  remote command modules.
- `crates/bonesdeploy/src/commands/skill.rs`, doctor command code, and client
  CLI integration tests under `crates/bonesdeploy/tests/`.
- BonesInfra Python tests and embedded-source tests that assert Python command
  availability or operation order.
- `e2e/src/project.rs`, `e2e/tests/setup/`, `e2e/tests/setup.rs`, and
  `e2e/README.md`.
- `README.md`, `CONTEXT.md`, `docs/ARCHITECTURE.md`,
  `docs/security/invariants.md`, status guidance, prompts, and
  `crates/bonesdeploy/assets/skill/{SKILL.md,commands.md,workflows.md}`.

### Decisions

- Server connection parsing reuses the project `.env` but ignores every field
  outside `HOST`, `SSH_USER`, and `PORT`. This avoids a second source of truth
  while making site configuration inaccessible to server provisioning.
- The server baseline is checked through `bonesremote doctor` host mode rather
  than a separate marker file. Doctor validates actual required artifacts and
  reports a damaged baseline, not merely a prior command invocation.
- `site apply` performs only site base provisioning. The Rust site setup
  orchestrator invokes services and runtime explicitly afterward, preserving
  their independent commands and making the observable sequence exact.
- Server readiness failure is handled before calling any site provisioning
  operation. This protects a host without a baseline from partial project
  identities, directories, repositories, or control-plane state.
- `status`, `manifest`, and `releases` are site-scoped and move under `site`.
  `rollback` remains top-level by the requested public contract.
- The command break is intentional: old flat inspection names, all `remote`
  variants, and hidden `guide` are removed without aliases. Root `setup` and
  `doctor` are retained as documented composition commands, so convenience
  workflows remain available without weakening the server/site boundary.
- Tests observe CLI behavior through crate integration tests and orchestration
  behavior through focused Python tests; they do not expose private production
  helpers solely for testing.

### Risks

- Moving global operations can omit a prerequisite currently provided by the
  combined setup path, leaving a site unable to build or a baseline doctor
  incomplete. Operation-order tests and host-doctor coverage mitigate this.
- A context refactor can leave a server operation reading `DeployContext` or a
  runner path tied to `AppConfig.server`, defeating the isolation boundary.
  Type signatures and server-context tests must reject that coupling.
- Removing command aliases breaks scripts and embedded documents that retain
  old syntax. Parser rejection tests and a repository-wide command-reference
  update make the clean break deliberate and visible.
- The existing E2E tests share an Incus server; retaining per-project host
  setup can hide baseline/site leakage. The harness must make the one-time
  server setup explicit before site scenarios.
- Host doctor failures can prevent a valid site from being created. Its checks
  must match only artifacts server provisioning establishes and use the same
  centralized paths and sudoers rendering contract.

### Validation

- Python tests prove `ServerContext` accepts the connection fields without
  parsing site configuration, `DeployContext` retains full site data, and the
  runner accepts both contexts through their shared connection contract.
- Focused provisioning tests monkeypatch each operation to prove server setup
  invokes every server operation exactly once and no site operation, while
  site setup invokes every site-base operation exactly once and no packages,
  firewall, global keys, BonesRemote installation, or global sudoers operation.
- BonesRemote doctor tests prove host mode validates every baseline artifact
  independently of `--site`, and existing site-doctor coverage still validates
  project boundaries.
- `bonesdeploy` integration tests prove all new parser/help routes, confirm
  removed flat and `remote` routes are rejected, verify local doctor is `site
  doctor --local`, and exercise server-missing guidance before site mutation.
- Skill tests prove the five-state next-step sequence and that a second project
  with a ready server is directed to `site setup`.
- E2E source and harness tests establish one `server setup` per shared server
  and one `site setup` per framework project. The E2E suite is not executed.
- Run targeted crate and Python test suites, then `cargo fmt`, `cargo clippy`,
  `shfmt -w .`, `ruff format .`, and `ruff check .`. Review the final diff and
  public help/documentation for old command references.


## Tasks

### Implementation

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

### Validation

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

### Completion

- [x] Review the final diff and generated public help to confirm root `setup`
  and `doctor` are only composition workflows, no obsolete flat, `remote`, or
  `guide` command remains, and no server flow reads site configuration.
- [x] Verify documentation and embedded guidance use `server` and `site`
  commands, and that every documented `site setup` sequence is readiness,
  base, services, runtime, doctor without Git, secrets, SSL, or deployment.

### Completion notes

Completed the server/site provisioning split, including isolated Python command
packages, complete host-mode BonesRemote baseline checks, scoped CLI coverage,
one-time shared-server E2E setup, and command-reference documentation updates.
The E2E target was compiled but its ignored scenarios were not executed.


## Clarification

### Trigger

The requested command break should not remove root `bonesdeploy setup`. The
existing convenience workflow remains useful when provisioning a fresh host
and project together, and all provisioning operations are idempotent.

### Decision

Retain root `bonesdeploy setup [--yes]` as a thin composition command. It runs
`bonesdeploy server setup --yes` first and then `bonesdeploy site setup --yes`.
The root command owns sequencing and failure propagation only; server and site
commands retain all provisioning responsibilities and can be run independently
or repeatedly. If server setup fails, root setup stops and does not invoke site
setup. If both delegated workflows succeed, root setup reports the same
completed site workflow and next-step guidance as site setup.

### Supersedes

This supersedes the earlier plan language that removed the root `setup`
command. It adds no change to the server/site ownership boundary, the server
readiness guard, or the exact site setup sequence.

### Effect on the record

`01-idea.md` now defines root setup as a retained convenience workflow and
limits command removal to obsolete flat inspection commands, `remote`, and
`guide`. `02-plan.md` assigns root setup to a thin Rust composition command and
specifies its server-then-site delegation and failure behavior. `03-tasks.md`
adds implementation and validation work for the composition path and updates
the public-help review requirement.


## Clarification

### Trigger

The root `bonesdeploy doctor` command should remain available as a convenience
workflow and should run both the host and project checks.

### Decision

Retain root `bonesdeploy doctor` as a thin diagnostic composition command. It
runs `bonesdeploy server doctor` followed by `bonesdeploy site doctor`, shows
both results, and returns failure if either check fails. It still invokes the
site check after a server-check failure so one command reports the complete
diagnostic state. The scoped doctor commands remain independently callable,
including `site doctor --local`.

### Supersedes

This supersedes the earlier plan language that removed the root `doctor`
command. It adds no change to server/site doctor ownership or to the server
readiness guard used by site setup.

### Effect on the record

`01-idea.md` now defines root doctor as a retained composed workflow and limits
command removal to the obsolete flat inspection commands, `remote`, and
`guide`. `02-plan.md` assigns root doctor to a thin server-then-site diagnostic
composition with aggregated failure. `03-tasks.md` adds implementation and
integration-test work for that composition and its public-help review.
