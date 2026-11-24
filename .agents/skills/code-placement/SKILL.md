---
name: code-placement
description: Decide where BonesDeploy code belongs and preserve dependency direction. Use before adding, moving, or substantially changing Rust or Python production code, command handlers, provisioning behavior, release behavior, integrations, persistence, presentation, embedded assets, or tests in this repository.
---

# Code Placement

Place code by the responsibility it owns, not by the caller that first needs it.

## Workflow

1. Name the behavior in one sentence.
2. Identify whether it runs on the workstation, during provisioning, or during remote deployment.
3. Find the existing owner below and inspect a sibling implementation.
4. Keep entry points thin: parse, validate, delegate, and present.
5. Keep dependencies flowing from entry points toward behavior and adapters.
6. Check coupled Rust/Python contracts when changing configuration, paths, or frameworks.
7. Put tests in the owning crate or Python package's test directory (see Test Placement) and test public behavior.

Do not create a new architectural area when an existing owner fits. Create a focused module when the behavior represents a distinct responsibility and the current file would otherwise mix reasons to change.

## Workspace Ownership

### `crates/bonesdeploy-core`

Own shared vocabulary and contracts used across processes:

- canonical configuration and validation: `src/config/`
- typed provisioning and remote transport models: `src/config/transport.rs`
- product-owned paths: `src/paths.rs`
- artifact and build contracts: `src/artifact.rs`, `src/build_contract.rs`

It must not depend on another workspace crate. Do not put workstation UX, provisioning operations, or remote execution here.

### `crates/bonesdeploy`

Own workstation behavior and public UX:

- CLI syntax and routing: `src/cli/`
- user-facing use-case coordination: `src/commands/`
- local artifact construction: `src/build/`
- SSH and Git integrations: `src/infra/`
- embedded static content: `src/assets/`
- framework questions, defaults, and generated environments: `src/frameworks.rs` and `src/frameworks/`
- prompts and terminal output: `src/ui/`

Command modules may coordinate these areas but must not become alternate SSH, Git, build, framework, or asset implementations.

### `crates/bonesinfra`

The Rust crate owns wheel embedding, project materialization, the project-scoped Python environment, and subprocess invocation. Python owns provisioning.

Within `python/src/bonesinfra/`:

- private Typer entry points and dispatch: `cli/`
- server and site provisioning plans: `provisioning/`
- typed request, context, and path data: `config/`
- PyInfra connection and execution mechanics: `pyinfra/`
- reusable Linux and language operations: `services/`
- framework-specific runtime and resource declarations: `frameworks/<name>/`
- project framework/custom loading: `project.py`
- resource inventory and inspection: `manifest.py`
- version-gated migrations: `patches/`

Do not put provisioning operations under `cli/`. The CLI selects a provisioning plan and passes it to the runner.

### `crates/bonesremote`

Own remote deployment execution:

- CLI syntax and routing: `src/cli/`
- command handlers: `src/commands/`
- release rules, lifecycle, locking, and state: `src/release/`
- runtime control, including systemd and Compose: `src/runtime/`
- shared-data transactions: `src/shared/`
- read-only operating-system inspection: `src/inspection/`
- synchronized site configuration persistence: `src/control_plane.rs`
- privilege checks: `src/privileges.rs`
- terminal output: `src/ui.rs`

`commands/` is an entry-point layer. Reusable release, runtime, shared-data, persistence, or inspection behavior must live in its owning module. Code under those modules must not import `crate::commands`.

All site mutation must use `release::SiteMutation`. Deployment stages extend `release/lifecycle/`; do not create a second deployment flow.

## Test Placement

All Rust test code belongs in the owning crate's crate-root `tests/` directory.
Production files under `src/` must not contain `#[cfg(test)]` modules, test
functions, test fixtures, test assertions, or test-only seams.

- Test library behavior through deliberate public APIs; test observable CLI
  behavior by executing the compiled binary (`CARGO_BIN_EXE_<name>`).
- Organize larger suites as modules below `tests/`; only files directly in
  `tests/` compile as integration-test targets. Shared support belongs under
  `tests/common/` or a focused support module below `tests/`.
- The crate-root `tests/` directory is Cargo's integration-test directory, not
  a workspace-level test package such as `tests/cleancode`.
- Do not reproduce production algorithms in integration tests. When important
  behavior has no existing boundary, restructure the production code around a
  deliberate, cohesive library API. Do not expose an arbitrary internal helper
  or promote a private item to `pub` solely so a test can call it.
- Do not add a test-support feature or workspace-level test crate when the
  standard integration-test boundary is sufficient.
- Use fixtures and factories when they clarify setup; keep them focused and
  isolate external state so tests stay repeatable and independent.
- A test that cannot be written through a public boundary is evidence the
  production boundary needs redesign, or that the assertion is coupled to an
  implementation detail and should be removed.

Command tests use a temporary working directory, isolated `HOME` and relevant
XDG directories, and explicit Git repositories and remotes when needed. They
capture status, output, and durable effects without mutating the test
process's current directory or global environment.

Full conventions: `docs/conventions/rust.md`.

## Dependency Direction

Use these directions:

```text
CLI -> commands -> release/runtime/shared/inspection/integrations
BonesDeploy commands -> bonesinfra subprocess boundary
BonesDeploy commands -> BonesRemote over SSH
BonesInfra CLI -> provisioning plans -> services/PyInfra
```

Avoid these directions:

```text
release/runtime/shared -> commands
provisioning -> CLI
config models -> PyInfra
bonesremote -> bonesinfra
bonesdeploy-core -> another workspace crate
```

Sibling domain modules may collaborate when the workflow genuinely spans them, but entry-point modules are never reusable service libraries.

## Common Decisions

| Behavior | Place it in |
| --- | --- |
| New public CLI option or subcommand | owning crate's `src/cli/args.rs` |
| Dispatch only | owning crate's `src/cli/dispatch.rs` |
| Workstation command workflow | `bonesdeploy/src/commands/` |
| Local source export, Docker build, or packaging | `bonesdeploy/src/build/` |
| SSH or Git mechanics | `bonesdeploy/src/infra/` |
| Embedded files | `bonesdeploy/src/assets/` |
| Shared config, validation, path, or wire contract | `bonesdeploy-core/src/` |
| Server or site provisioning workflow | `bonesinfra/provisioning/` |
| Reusable provisioning operation | `bonesinfra/services/` |
| PyInfra connection/execution behavior | `bonesinfra/pyinfra/` |
| Release transition or cleanup rule | `bonesremote/src/release/` |
| Service or Compose runtime control | `bonesremote/src/runtime/` |
| Shared import/environment transaction | `bonesremote/src/shared/` |
| Read-only OS query | `bonesremote/src/inspection/` |
| Human-facing formatting | owning `ui` or CLI output module |

## Coupled Changes

- Keep `bonesdeploy-core/src/paths.rs` and Python `config/paths.py` aligned.
- Framework support commonly touches Rust framework definitions, Python framework provisioning, and embedded assets.
- Rust alone parses the root `.env`; Python receives `ProvisioningRequest`, and BonesRemote receives `RemoteDeploymentConfig`.
- Provisioning belongs to BonesInfra. Release receipt, activation, rollback, and pruning belong to BonesRemote.

## Avoid

- Do not use `commands/`, `infra/`, `shared/`, or `utils` as catch-all locations.
- Do not place reusable behavior in dispatch files.
- Do not import command handlers from domain modules.
- Do not duplicate product paths, config keys, environment names, state stores, or external-system adapters.
- Do not copy an existing placement mistake merely because it is nearby; follow the ownership rules above and remove the inversion when the requested change exposes it.
