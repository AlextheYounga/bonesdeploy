# Idea

## Request

Define a durable architectural refactor program for BonesDeploy. The program will
move important responsibilities into clear, reusable building blocks with small
public APIs, private implementation details, explicit collaborators, and one
extension mechanism per family of variations.

This Acta record defines the target architecture and the order of the child
changes. The initial implementation slice is limited to restoring the branch
build, correcting architecture documentation, and implementing the Rust Framework
boundary; it does not implement every child refactor in one change.

## Problem

The repository has several useful concepts, but their boundaries are inconsistent.
Commands and sibling modules can bypass existing wrappers and coordinate internal
details directly.

The current problems are concrete:

- framework behavior uses string dispatch, duplicate registries, and public module
  internals;
- direct Git and SSH processes bypass existing Rust wrappers;
- dotenv parsing and project configuration are implemented in multiple places;
- update and migration behavior is split across Rust and Python, including a dead
  Rust migration alongside the live Python patch;
- Python framework runtimes repeat common provisioning orchestration;
- deployment lifecycle coordination is concentrated in a large command function;
- state storage and mutation guards are reachable through side doors;
- doctor, status, service, lifecycle, and build code duplicate inspectors and path
  derivation;
- architecture documentation describes the pre-decentralization system.

These are ownership and boundary problems, not merely duplication problems.

## Definitions

**Building block:** A named responsibility with a small public API, private
implementation, explicit collaborators, and a documented extension mechanism.

**Public concept:** The API that callers in another responsibility may use. It may
be a concrete Rust type, module, function set, Python class, or registry.

**Side door:** A caller path that bypasses an established public concept, such as a
direct `git` process outside `infra/git.rs`, a direct state-file write outside the
state store, or a direct import of a framework implementation module.

**Child change:** A separately planned and reviewed implementation change that
implements one architectural boundary from this program. A child change must have
its own settled Acta `01-idea.md`, `02-plan.md`, and `03-tasks.md` before code is
modified.

**Canonical owner:** The one existing layer, module, process, or concept that owns
a responsibility. Callers delegate to the canonical owner and do not reproduce
its implementation details.

## Target Architecture

The program establishes these canonical ownership groups. The names identify
responsibilities already present in the repository; they do not require a new type
when an existing module already provides the correct boundary.

| Responsibility | Canonical concept or boundary | Existing anchor |
| --- | --- | --- |
| Project configuration and `.env` | Project configuration boundary | `bonesdeploy-core/src/config.rs`, `bonesdeploy/src/config.rs`, Python `config/context.py` |
| Framework selection behavior | Rust `Framework` concept | `bonesdeploy/src/frameworks.rs` |
| Framework provisioning | Python framework package and project infrastructure | `bonesinfra/project.py`, `frameworks/<name>/` |
| Language installation | `LanguageRuntime` | `bonesinfra/services/languages/` |
| Database/cache services | `RuntimeService` registry | `bonesinfra/services/runtime/` |
| Git operations | Git boundary | `bonesdeploy/src/infra/git.rs` |
| SSH operations | SSH boundary | `bonesdeploy/src/infra/ssh.rs`, Python `pyinfra/runner.py` |
| Secrets | GPG/secrets boundary | `bonesdeploy/src/commands/secrets/` |
| Infrastructure updates | Update flow plus `Patch` registry | `commands/update/`, Python `patches/registry.py` |
| Provisioning execution | BonesInfra command plans and runner | `bonesinfra/cli/`, `pyinfra/runner.py` |
| Deployment mutation authorization | `SiteMutation` | `bonesremote/src/release/site_mutation.rs` |
| Deployment state | `SiteState` store | `bonesremote/src/release/state/` |
| Deployment lifecycle | Lifecycle stages and phase model | `bonesremote/src/release/lifecycle/`, `DeploymentPhase` |
| Health inspection | Doctor and shared inspectors | `bonesdeploy/commands/doctor.rs`, `bonesremote/commands/doctor/` |

## Desired Outcome

When this program is decomposed into approved child changes:

1. `docs/ARCHITECTURE.md` and `docs/architecture/reference.md` identify each
   canonical owner and match the post-decentralization code.
2. Each child change has one settled responsibility, one chosen boundary, concrete
   callers to migrate, and focused validation.
3. Commands delegate through public concepts instead of reaching into internals.
4. Existing variation mechanisms are extended instead of parallel registries being
   created.
5. Side-door searches and visibility checks verify important boundaries.
6. No child change introduces speculative managers, containers, generic service
   layers, or traits without a concrete substitution requirement.

## Scope

This parent change includes:

- settling the architecture vocabulary and canonical ownership map;
- correcting architecture documentation for the current `.env`/`infra/` system;
- identifying existing side doors and the child change that closes each one;
- defining the dependency order for child implementation changes;
- creating separate Acta planning records for each child change;
- the authorized initial bootstrap/documentation/Framework implementation slice.
- the authorized project configuration implementation slice.

## Constraints

- The initial slice may modify the manifest command and Rust Framework callers as
  recorded in clarification 11; remaining child changes are implemented separately.
- Child changes are implemented separately and reviewed independently.
- The project configuration slice preserves the flat `.env` wire format and does
  not consume the deferred `bonesinfra_input` or nested `App` serde contracts.
- Existing concepts are strengthened before new abstractions are introduced.
- `bonesdeploy-core` remains a leaf crate; `bonesremote` does not depend on
  `bonesinfra`.
- Provisioning remains owned by BonesInfra and deployment execution remains owned
  by BonesRemote.
- The repository revision remains the deployment unit.
- Child plans must preserve security, atomic state writes, lifecycle gates, and
  observable CLI behavior unless they explicitly record a bug fix.
- E2E tests are not part of ordinary child-change validation.

## Exclusions

- Implementing the child refactors is excluded from this parent documentation
  change.
- Replacing every function with an object is excluded.
- Speculative dependency injection, service containers, generic managers, and
  open-ended trait hierarchies are excluded.
- Merging the Rust CLI, BonesInfra, and BonesRemote into one layer is excluded.
- Changing product behavior without a child plan and explicit decision is excluded.
