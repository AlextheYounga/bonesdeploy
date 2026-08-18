# Plan

## Current Behavior

The repository already has named concepts, but the current boundaries are not
consistently enforced.

### Existing concepts and observed defects

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

### Documentation state

The architecture documents still contain pre-decentralization references to
`bones.toml`, `.bones/infra/`, removed config-repository commands, and the old
framework dispatch contract. This parent change must correct those references
before child plans are treated as authoritative.

## Chosen Approach

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

## Project Configuration Slice

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

## Responsibilities And Boundaries

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

## Child Plan Contracts

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

## Affected Documentation

The first child change updates:

- `docs/ARCHITECTURE.md` ownership map, reusable concepts, extension points, and
  post-decentralization paths;
- `docs/architecture/reference.md` framework, configuration, patch, update,
  provisioning, doctor, and deployment descriptions;
- this parent Acta record's references to the settled architecture.

The initial implementation slice updates the manifest command, architecture
documentation, and Rust Framework callers. Later child changes remain separate and
must not silently redefine these boundaries.

## Decisions

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

## Risks

- The parent may become a second general architecture document if it records code
  details instead of child-change boundaries; its scope is limited to program
  definition and decomposition.
- A child plan may accidentally recreate a side door unless it identifies all
  callers and reduces internal visibility after migration.
- Cross-layer contracts can drift across Rust, Python, and repository revisions;
  child validation must test those boundaries explicitly.
- Deployment and migration children can cause data loss; their plans must preserve
  atomic writes, markers, revision consistency, and lifecycle gates.

## Validation

- Verify `01-idea.md`, `02-plan.md`, and `03-tasks.md` describe the same parent
  change and use the same defined terms.
- Verify the parent contains no open questions, alternatives, placeholder
  decisions, or speculative implementation APIs.
- Verify every child boundary has one owner, one dependency position, and a stated
  child-plan deliverable.
- Verify architecture-document corrections are listed as concrete first-child work.
- Run `git diff --check` and review the final planning diff.
- Do not run e2e tests for this documentation-only parent change.

## Initial Implementation Result

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
