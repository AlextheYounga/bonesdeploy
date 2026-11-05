# Plan

## Current behavior

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

## Intended behavior

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

## Approach

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

## Responsibilities and boundaries

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

## Affected areas

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

## Decisions

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

## Risks

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

## Validation

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
