# Framework Abstractions

<!-- Merged from refactor/framework-abstractions -->

## Idea

### Request

Define a durable architectural refactor program for BonesDeploy. The program will
move important responsibilities into clear, reusable building blocks with small
public APIs, private implementation details, explicit collaborators, and one
extension mechanism per family of variations.

This Acta record defines the target architecture and the order of the child
changes. The initial implementation slice is limited to restoring the branch
build, correcting architecture documentation, and implementing the Rust Framework
boundary; it does not implement every child refactor in one change.

### Problem

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

### Definitions

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

### Target Architecture

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

### Desired Outcome

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

### Scope

This parent change includes:

- settling the architecture vocabulary and canonical ownership map;
- correcting architecture documentation for the current `.env`/`infra/` system;
- identifying existing side doors and the child change that closes each one;
- defining the dependency order for child implementation changes;
- creating separate Acta planning records for each child change;
- the authorized initial bootstrap/documentation/Framework implementation slice.
- the authorized project configuration implementation slice.
- the authorized Rust Git and SSH integration implementation slice.

### Constraints

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

### Exclusions

- Implementing the child refactors is excluded from this parent documentation
  change.
- Replacing every function with an object is excluded.
- Speculative dependency injection, service containers, generic managers, and
  open-ended trait hierarchies are excluded.
- Merging the Rust CLI, BonesInfra, and BonesRemote into one layer is excluded.
- Changing product behavior without a child plan and explicit decision is excluded.


## Plan

### Current Behavior

The repository already has named concepts, but the current boundaries are not
consistently enforced.

#### Existing concepts and observed defects

| Concept | Current implementation | Boundary defect |
| --- | --- | --- |
| Project configuration | Rust `config.rs`, Rust save code, Python `DeployContext` | Multiple dotenv parsers and stale TOML-shaped structures |
| Framework | Rust dispatch module plus per-framework modules and Python packages | String dispatch, asset/defaults reach-through, duplicate registries |
| Git | `infra/git.rs` | `doctor.rs` and update code shell out directly |
| SSH | `infra/ssh.rs` and Python pyinfra runner | Update release code directly invokes `ssh` |
| Secrets | GPG module and secrets commands | Command layer still assembles remote shell details |
| Updates/migrations | Rust update modules and Python `Patch` registry | Responsibilities split; dead Rust migration duplicates Python migration |
| Provisioning | Python command plans and framework runtimes | Repeated workflow and overlapping custom-hook paths |
| Deployment state | `SiteState`, state store, `DeploymentLock`, `SiteMutation` | Crate-wide state access and mutation-guard bypasses |
| Lifecycle | Lifecycle stage modules and `run_staged_deployment()` | Phase advancement and failure handling concentrated in a command |
| Doctor/inspection | Local/remote doctor, status, service, lifecycle helpers | Repeated account, process, systemd, script, and path inspection |

The repository investigation identified the following concrete side doors:

- direct `git` in `crates/bonesdeploy/src/commands/doctor.rs`;
- direct `ssh` in `crates/bonesdeploy/src/commands/update/release.rs`;
- multiple `.env` and `TEMPLATE` parsers;
- direct per-framework defaults access from `infra/assets/frameworks.rs`;
- direct state-store access from release commands;
- `SiteMutation` bypasses in `release/kill.rs` and configuration bypasses in
  `commands/doctor/site.rs`;
- duplicated process, systemd, account, script, and path inspectors;
- dead `commands/migrate.rs` beside the live Python `0003-project-infra` patch.

The detailed settled findings are recorded in the numbered clarification documents
in this directory:

- `06-framework-boundary-findings-clarity.md`
- `07-integration-boundary-findings-clarity.md`
- `08-deployment-boundary-findings-clarity.md`
- `09-provisioning-update-findings-clarity.md`
- `10-architecture-document-drift-clarity.md`

Those clarifications are part of the current record, not alternative plans.

#### Documentation state

The architecture documents still contain pre-decentralization references to
`bones.toml`, `.bones/infra/`, removed config-repository commands, and the old
framework dispatch contract. This parent change must correct those references
before child plans are treated as authoritative.

### Chosen Approach

This is a parent planning change. It establishes one architecture map and creates
one child Acta change per implementation boundary. It does not combine all code
refactors into one branch or one implementation task.

The child changes are ordered as follows:

1. **Architecture documentation and ownership map** — correct the current docs and
   record the public concepts and side-door rules used by all later plans.
2. **Project configuration boundary** — settle the `.env` parser/writer and project
   identity boundary used by Rust and Python.
3. **Git, SSH, and secrets boundaries** — migrate callers through existing external
   integration wrappers.
4. **Infrastructure updates and migrations** — unify release synchronization and
   the Python patch registry; remove the dead migration path.
5. **Provisioning composition** — strengthen project infrastructure, provisioning,
   language runtime, service, manifest, and framework-runtime boundaries.
6. **Framework concept** — implement the typed Rust framework front door and its
   private implementation contract.
7. **Deployment and lifecycle** — close mutation/state side doors and place phase,
   failure, rollback, and cleanup behavior behind the deployment boundary.
8. **Doctor and shared inspection** — centralize reusable inspectors and make
   doctor/reporting consume them.
9. **Final boundary audit** — verify visibility, callers, documentation, and static
   side-door checks across the completed child changes.

Each child plan must identify its exact owner, affected callers, chosen public API,
private implementation boundary, tests, and completion condition. Later child
plans may depend on earlier concepts but may not silently redefine them.

### Project Configuration Slice

The Rust configuration module remains the owner of the flat `.env` contract. Its
`project_env` constants define every persisted key, and its `load()` and `save()`
functions form the parse/write pair. The CLI configuration module re-exports that
writer while retaining only local CLI helpers.

Python retains its existing dotenv parser in `config/context.py`, but the parser is
the single reader used by both `DeployContext` and framework selection. Python's
`config/keys.py` owns the corresponding exclusion vocabulary so unknown framework
values continue to flow into runtime data without duplicating key literals.

The slice deliberately does not remove `bonesinfra_input`, change `DeploymentPaths`,
or flatten the nested `App` serialization used by build environment derivation.

### Responsibilities And Boundaries

The parent ownership rules are:

| Responsibility | Owner | Required caller behavior |
| --- | --- | --- |
| `.env` loading and writing | Project configuration boundary | Use the owner; do not parse independently |
| Rust framework selection | `Framework` | Parse once, call methods, do not match strings or import modules |
| Python framework provisioning | Framework package/project infrastructure | Use `LanguageRuntime` and `RuntimeService` |
| Language installation | `LanguageRuntime` | Select and install through the runtime contract |
| Service provisioning | `RuntimeService` registry | Resolve and provision through the registry |
| Git | `infra/git.rs` | Do not spawn `git` from commands |
| SSH | `infra/ssh.rs` / pyinfra runner | Do not spawn `ssh` outside the wrapper |
| Infrastructure migration | Python `Patch` registry | Do not maintain a parallel migration path |
| Site mutation | `SiteMutation` | Do not assemble locks/configuration independently |
| Site state | `SiteState` store | Do not touch state files directly |
| Lifecycle | Deployment/release lifecycle | Do not advance phases or implement rollback in commands |
| Inspection | Doctor/shared inspectors | Do not duplicate probes in unrelated commands |

### Child Plan Contracts

Every child Acta plan must contain:

- a settled request and problem;
- definitions for any new domain term or boundary;
- an observable desired outcome;
- positive scope, constraints, and exclusions;
- repository-grounded current behavior;
- one chosen implementation approach;
- explicit responsibilities and affected callers;
- decisions, risks, and validation;
- concrete tasks with completion conditions.

Child implementation must not begin until its plan is reviewed and approved.

### Affected Documentation

The first child change updates:

- `docs/ARCHITECTURE.md` ownership map, reusable concepts, extension points, and
  post-decentralization paths;
- `docs/architecture/reference.md` framework, configuration, patch, update,
  provisioning, doctor, and deployment descriptions;
- this parent Acta record's references to the settled architecture.

The initial implementation slice updates the manifest command, architecture
documentation, and Rust Framework callers. Later child changes remain separate and
must not silently redefine these boundaries.

### Decisions

1. **Parent plus child changes.** The repository-wide direction is recorded here;
   implementation is split into independently reviewable child changes.
2. **Existing concepts first.** Git, SSH, `LanguageRuntime`, `RuntimeService`,
   `Patch`, `SiteMutation`, `SiteState`, and lifecycle phases are strengthened
   before new concepts are introduced.
3. **Concrete boundaries over abstraction count.** A module, enum, struct, or
   function set is acceptable when it gives ownership and hides details. Traits
   are not required by this plan.
4. **Commands delegate.** Commands remain thin entry points that validate, select a
   concept, delegate, and report.
5. **Documentation is a prerequisite.** The ownership map must match the current
   `.env`/`infra/` system before child implementation plans become authoritative.

### Risks

- The parent may become a second general architecture document if it records code
  details instead of child-change boundaries; its scope is limited to program
  definition and decomposition.
- A child plan may accidentally recreate a side door unless it identifies all
  callers and reduces internal visibility after migration.
- Cross-layer contracts can drift across Rust, Python, and repository revisions;
  child validation must test those boundaries explicitly.
- Deployment and migration children can cause data loss; their plans must preserve
  atomic writes, markers, revision consistency, and lifecycle gates.

### Validation

- Verify `01-idea.md`, `02-plan.md`, and `03-tasks.md` describe the same parent
  change and use the same defined terms.
- Verify the parent contains no open questions, alternatives, placeholder
  decisions, or speculative implementation APIs.
- Verify every child boundary has one owner, one dependency position, and a stated
  child-plan deliverable.
- Verify architecture-document corrections are listed as concrete first-child work.
- Run `git diff --check` and review the final planning diff.
- Do not run e2e tests for this documentation-only parent change.

### Initial Implementation Result

The authorized initial slice is complete:

- the committed `commands/manifest.rs` conflict was resolved in favor of the
  post-decentralization `.env` implementation;
- both architecture documents now describe the current `.env`/`infra/` system;
- Rust framework identity is represented by `Framework` with eight wire values,
  centralized dispatch, private per-framework modules, explicit custom fallback,
  and migrated callers;
- update synchronization preflights managed conflicts before mutation;
- focused tests cover framework parsing, defaults, custom behavior, Rails runtime
  selection, asset identity, secrets validation, and update conflict safety.
- project configuration validation covers the Rust `.env` round trip, all core
  configuration tests, Python key/parser reuse, and quoted framework selection.

### Integration Slice

The existing Rust integration modules remain the owners of external Git and SSH
processes. `infra::git` exposes branch inspection and repository cloning for local
callers, while `infra::ssh` remains the owner of remote command execution. Doctor
uses the Git boundary for branch checks, update uses it for release-source cloning,
and remote version discovery uses the SSH connection and command boundary. Python
bare-repository setup, secrets shell composition, and the dead `bonesinfra_input`
contract remain deferred.


## Tasks

### Parent Planning Record

- [ ] Review `01-idea.md`, `02-plan.md`, and `03-tasks.md` together and remove any
  term, scope boundary, or decision that is not settled in all three files.
- [ ] Verify the canonical ownership table names existing repository anchors and
  does not introduce unapproved implementation types.
- [ ] Verify the child changes are ordered by dependency and that no child silently
  changes an earlier boundary.
- [ ] Read clarifications `06` through `10` into the relevant child plans so the
  exact repository findings are not replaced by generic architectural language.

### First Child: Architecture Documentation

- [x] Create the child Acta change for architecture documentation and ownership
  mapping.
- [x] In that child plan, inventory and correct stale `.bones/`, `bones.toml`,
  removed-command, patch, and framework-contract references in
  `docs/ARCHITECTURE.md` and `docs/architecture/reference.md`.
- [x] In that child plan, document the current `.env`, `infra/`, committed revision,
  `infra/.framework`, and `infra/custom` boundaries.
- [ ] Complete the child plan review before creating implementation tasks for later
  slices.

### Initial Implementation Slice

- [x] Resolve the committed `commands/manifest.rs` conflict with the current
  post-decentralization implementation.
- [x] Implement `Framework` parsing/display, dispatch, defaults routing, private
  per-framework modules, and explicit `Custom` behavior.
- [x] Migrate init, scaffold, secrets, prompts, and update synchronization callers.
- [x] Preserve update safety by preflighting managed-core conflicts before copying
  deployment or managed files.
- [x] Add focused regression tests for custom persistence, invalid templates, Rails
  versions, asset identity, and unchanged trees after update conflicts.

### Project Configuration Slice

- [x] Centralize the complete flat `.env` key vocabulary in the core configuration
  module and use it for loading and writing.
- [x] Move the Rust `.env` writer beside the core loader and preserve the CLI
  re-export used by existing callers.
- [x] Add Python configuration-key constants and use the shared dotenv parser for
  framework selection.
- [x] Add round-trip coverage for all persisted configuration fields and quoted
  Python framework selection.
- [x] Leave the dead `bonesinfra_input` contract and nested `App` serde coupling for
  their dependency-ordered child boundaries.

### Integration Slice

- [x] Add Git-boundary operations for local branch inspection and release-source
  cloning.
- [x] Route doctor branch checks and update source cloning through the Git boundary.
- [x] Route remote version discovery through the SSH connection and command
  boundary.
- [x] Preserve Python bare-repository setup, secrets shell composition, and the
  dead `bonesinfra_input` contract for later child boundaries.
- [x] Add focused wrapper and caller regression coverage.

### Child Plan Decomposition

The requested implementation was completed directly as one coordinated change;
separate implementation slices and additional Acta child records were
intentionally not created.

- [x] Implement the project configuration boundary directly, including the `.env` parser/writer,
  project identity callers, duplicate parsers,
  and the dead `bonesinfra_input` contract.
- [x] Implement Git, SSH, and secrets boundaries directly, including protected delivery,
  the direct `git`/`ssh` callers, and Python bare-repository setup.
- [x] Implement infrastructure update and migration cleanup directly,
  including managed synchronization, `0003-project-infra`, patch markers, and the
  dead Rust migration.
- [x] Implement provisioning composition directly, including
  project infrastructure, language runtimes, runtime services, manifest, repeated
  framework runtime workflows, custom-hook overlap, runner access, and paths.
- [x] Implement the Rust/Python Framework contract directly, including
  centralized validation, defaults, permission defaults, identity,
  private modules, asset reach-through, and materialization boundaries.
- [x] Implement Deployment, SiteMutation, SiteState, and ReleaseLifecycle boundaries directly,
  including kill/recover/doctor bypasses, state
  visibility, duplicated inspectors, and lifecycle invariants.
- [x] Implement Doctor and shared inspection collaborators directly.
- [x] Complete the cross-boundary static checks and cleanup after the preceding changes.

### Parent Validation

- [x] Confirm the parent docs contain no unanswered questions, competing approaches,
  placeholder decisions, or speculative public APIs.
- [x] Confirm every named child change has one responsibility, one owner, and one
  dependency position; separate Acta plan deliverables were intentionally omitted.
- [x] Confirm the parent scope was expanded by explicit request to include implementation of all child changes.
- [x] Run `git diff --check`.
- [x] Review the final planning diff for consistency with the Acta skill.

### Completion

- [x] Mark this parent record complete after the coordinated implementation and
  verification of all named boundaries.
- [x] Record the settled implementation order and deliberately retained recovery
  exception in completion notes.

### Completion notes

- The complete refactor was implemented directly in dependency order without
  creating additional slice records, per the explicit request to avoid more slices.
- The deliberate exception is malformed-state recovery: it acquires the raw lock
  and accesses the store so it can quarantine invalid state before configuration
  validation is possible.
- Validation passed: `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo fmt --all -- --check`, `shfmt -d .`, 71 `bonesdeploy` binary unit tests,
  97 `bonesremote` unit tests, `bonesdeploy-core` tests, and targeted `bonesdeploy`
  CLI integration tests.
- E2E tests were not run by design.
- Project configuration validation passed: the focused Rust parser tests, all
  `bonesdeploy-core` tests, `ruff check .`, and the full 399-test Python suite.
- Integration validation passed: Git/SSH wrapper tests, targeted `bonesdeploy`
  tests, and static review found no direct `git` or `ssh` process calls in the
  migrated callers. E2E tests remain intentionally unrun.


## Clarification

### Trigger

The framework-only plan did not represent the requested change. The target is a
repository-wide move toward clear, reusable building blocks for framework
selection, configuration, Git, SSH, provisioning, updates, migrations, doctor,
deployment state, and lifecycle behavior.

### Decision

The existing framework-abstractions record is superseded by an umbrella
architectural refactor plan. `Framework` remains an implementation slice, not the
overall objective.

The refactor will identify major responsibilities and canonical owners, strengthen
existing concepts before introducing new abstractions, give each concept a small
public API with private collaborators, migrate callers through those concepts,
close direct reach-through, preserve crate and Rust/Python/revision boundaries, and
proceed as sequenced reviewable slices rather than one all-at-once rewrite.

The architecture documentation is included in the first slice because it must
describe the post-decentralization system and the target concepts.

### Supersedes

This supersedes the previous framework-only request, scope, intended outcome, and
task sequence in this Acta record. It does not discard the concrete framework
findings; it places them under the broader building-block program.

### Effect on the record

- `01-idea.md` now defines the repository-wide architectural goal, target concepts,
  boundaries, constraints, and exclusions.
- `02-plan.md` now describes the current cross-crate architecture, side doors,
  composition rules, and sequenced refactor approach.
- `03-tasks.md` now sequences configuration, integrations, infrastructure updates,
  provisioning, framework, deployment, doctor, and side-door closure work.
- Architecture documentation updates are explicit implementation tasks, including
  correction of stale `.bones`/`bones.toml`/removed-command references.


## Clarification

### Trigger

The umbrella architecture record contained broad implementation tasks and
provisional concepts. Acta requires a settled change, one chosen approach, and
concrete tasks with clear completion conditions.

### Decision

This Acta record is narrowed to defining and decomposing the architectural
refactor program. It does not claim to implement every repository-wide refactor.
Each implementation boundary will receive its own reviewed child Acta change.

The parent record contains only the settled architecture vocabulary, canonical
owners, child-change order, child-plan requirements, and validation of the planning
record itself.

### Supersedes

This supersedes the previous umbrella implementation roadmap in `01-idea.md`,
`02-plan.md`, and `03-tasks.md`, which treated all child refactors as one broad
implementation plan.

### Effect on the record

- `01-idea.md` defines a documentation-and-decomposition parent change with settled
  scope and exclusions.
- `02-plan.md` contains the repository-grounded architecture inventory, ownership
  rules, chosen parent/child approach, and child-plan contract.
- `03-tasks.md` contains concrete parent planning tasks and explicit child-plan
  deliverables rather than generic implementation tasks.
- Implementation waits for the relevant child Acta plan and human approval.


## Clarification

### Trigger

The framework investigation found more boundary defects than the parent summary
listed. The concrete findings must remain available to the Framework child plan.

### Decision

The Rust Framework child change owns all Rust-side framework selection behavior,
including defaults and asset selection, not only questions and environment
examples.

The current framework boundary includes:

- `crates/bonesdeploy/src/frameworks.rs` string dispatch for questions,
  validation, configuration, environment examples, and build examples;
- `infra/assets/frameworks.rs::framework_defaults()` as an additional dispatch
  path that reaches per-framework defaults directly;
- `FrameworkDefaults` and `PermissionDefault` as part of the Rust framework
  contract;
- centralized `frameworks.rs::validate_answers()` driven by question schemas,
  rather than independent per-framework validators;
- `Runtime.template: String`, `FrameworkSelection.template: Option<String>`,
  embedded asset directories, and Python `BUILTIN_FRAMEWORKS` as separate identity
  sources;
- eight wire identities, including `custom`;
- Python materialization into `infra/.framework` and `infra/custom`,
  with managed core and user-owned custom content.

The child plan must close the defaults/asset reach-through, define the typed Rust
front door, preserve the `.env` wire contract, and test Rust/Python/materialization
consistency. It must not move Python provisioning ownership into Rust.

### Supersedes

This adds concrete framework findings to the parent architecture inventory. It
supersedes the narrower assumption that framework work only covered five dispatch
functions.

### Effect on the record

- `02-plan.md` identifies framework defaults, centralized validation, eight wire
  identities, and the materialization boundary as part of the current behavior.
- `03-tasks.md` requires the Framework child plan to cover defaults, assets,
  identity registries, and cross-layer materialization tests.


## Clarification

### Trigger

The integration audit found existing Git, SSH, and secrets wrappers that callers
bypass, plus a dead Rust-to-Python input contract.

### Decision

The integration child change will strengthen existing boundaries rather than add
parallel adapters.

Concrete bypasses are:

- `crates/bonesdeploy/src/commands/doctor.rs` directly invokes `git` instead of
  using `infra/git.rs`;
- `crates/bonesdeploy/src/commands/update/release.rs` directly invokes `ssh`
  instead of using `infra/ssh.rs`;
- Python setup directly initializes bare Git repositories through pyinfra shell
  commands because no Python Git concept exists for that responsibility;
- the secrets command assembles remote temporary-file shell commands outside its
  GPG/secrets boundary;
- Rust builds `bonesinfra_input` JSON in `commands/remote/data.rs`, but Python
  does not read that stdin contract and derives its context from `.env` instead.

The child plan must choose one owner for each behavior, migrate callers, remove
dead contracts, and add static checks against direct process bypasses. It must
preserve the Rust `bonesinfra::run*` subprocess boundary and Python's pyinfra
runner boundary.

### Supersedes

This adds concrete integration findings to the parent summary and supersedes the
generic statement that Git and SSH wrappers merely need strengthening.

### Effect on the record

- `02-plan.md` records the exact direct-process callers and dead JSON contract.
- `03-tasks.md` requires the integration child plan to address those callers and
  contracts explicitly.


## Clarification

### Trigger

The deployment audit found strong existing primitives but important bypasses and
duplicated lifecycle helpers that must be preserved in the Deployment child plan.

### Decision

The Deployment child change will strengthen `SiteMutation`, `SiteState`,
`DeploymentSnapshot`, `DeploymentPhase`, and the existing lifecycle modules into a
clear public deployment boundary.

Concrete findings are:

- `release/commands/deploy/lifecycle.rs::run_staged_deployment()` coordinates phase
  advancement and several abort, activation-failure, cleanup, and rollback paths
  inline;
- `release/kill.rs` manually acquires a lock and adopts `Bones::for_site(site)`,
  bypassing normal validated site configuration;
- `release/recover.rs` accesses the lock and state store directly;
- `commands/doctor/site.rs` uses convention-derived `Bones::for_site()` rather
  than validated site configuration;
- `release::state` exposes crate-wide read/write functions and callers directly
  manipulate active and staged state;
- process-start parsing, systemd requirement parsing, account/group parsing,
  numbered-script listing, prepare-input streaming, and nginx path derivation are
  duplicated across lifecycle and command modules;
- status, service, release listing, and doctor independently derive release,
  service, account, and path information.

The child plan must preserve atomic state writes, confused-deputy validation,
mutation locking, preflight gating, activation semantics, revision consistency,
rollback behavior, and cleanup outcomes while closing these side doors.

### Supersedes

This adds the concrete deployment findings that were compressed out of the parent
summary and supersedes the generic phrase "strengthen deployment boundaries."

### Effect on the record

- `02-plan.md` identifies the exact deployment bypasses and duplicated helpers.
- `03-tasks.md` requires the Deployment child plan to cover mutation, state,
  lifecycle, and inspection relationships with these invariants intact.


## Clarification

### Trigger

The provisioning and update audit found concrete existing contracts and duplicate
paths that were compressed out of the parent record.

### Decision

The provisioning child change will preserve and strengthen the existing
`LanguageRuntime`, `RuntimeService`, Manifest, project materialization, and pyinfra
runner concepts.

The update child change will make the Python `Patch` registry the sole migration
mechanism and give managed infrastructure synchronization one owner.

Concrete findings are:

- Python framework `runtime.py` files repeat the same setup, shared-directory,
  application, custom-hook, and service-start workflow;
- built-in `custom.py` hooks overlap with materialized `infra/custom`
  composition;
- direct pyinfra global `host` access escapes the runner boundary;
- infrastructure paths are scattered as module constants instead of consistently
  using `DeploymentPaths`;
- `RuntimeService.get_service()` exits the process on lookup failure;
- the live patch is `0003-project-infra`, introduced in version `0.8.0`, and its
  remote scope writes markers only;
- `crates/bonesdeploy/src/commands/migrate.rs` is a dead Rust duplicate of the
  live Python migration;
- update synchronization copies managed `infra/.framework` and deployment
  content, refuses modified-core conflicts, and preserves custom content.

The child plans must preserve language/service extension points, managed-core versus
custom ownership, patch markers, idempotency, and revision consistency.

### Supersedes

This adds the concrete provisioning and migration findings that were compressed out
of the parent summary and supersedes the generic provisioning/update descriptions.

### Effect on the record

- `02-plan.md` identifies the repeated workflow, custom-hook overlap, runner/path
  bypasses, live patch, dead migration, and managed sync behavior.
- `03-tasks.md` requires separate child plans to address provisioning composition
  and update/migration ownership.


## Clarification

### Trigger

The architecture-document audit found precise post-decentralization drift that must
be preserved for the documentation child change.

### Decision

The architecture documentation child change will correct these documented facts:

- project configuration is root `.env`, not `.bones/bones.toml`;
- project infrastructure is `infra/`, including
  `infra/.framework/`, `infra/custom/`, and `infra/secrets/`;
- the deployment unit is a committed repository revision;
- removed config-repository, import/receive, push, pull, and deploy-on-push flows
  are not current architecture;
- the Rust Framework contract includes centralized validation, defaults, build
  environment generation, and permission defaults;
- the Python framework contract uses materialized core/custom packages;
- the only current migration is `0003-project-infra` at version `0.8.0`, with
  remote marker behavior;
- `SiteMutation` has its actual constructors and validated-config location;
- deployment state is stored in `deployment.json`;
- local and remote doctor flows reflect current layout and command ownership;
- `RuntimeBackend`, `Runtime.permissions`, manifest socket artifacts, current
  language-runtime signatures, and current service registry behavior are documented.

The child change will update both `docs/ARCHITECTURE.md` and
`docs/architecture/reference.md`; the parent and other child plans will link to
those corrected concepts rather than preserving conflicting summaries.

### Supersedes

This adds the exact architecture-document corrections that were previously described
only as stale documentation.

### Effect on the record

- `02-plan.md` identifies the documentation drift as a concrete current behavior.
- `03-tasks.md` requires the architecture child plan to cover these exact sections
  and post-decentralization facts.
