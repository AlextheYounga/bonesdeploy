# Site Delete Command

<!-- Merged from feature/deletesite -->

## Idea

### Request

Provide `bonesdeploy site delete` as the inverse of site setup: permanently
remove one site's remote resources without introducing a separate deleted-site
lifecycle.

### Problem

The current command persists a deletion plan, decommissioning state, and a
verified tombstone. Those states block mutations, remain registered after
successful deletion, and require setup reactivation. This machinery has caused
deletion retries and fresh setup to fail even after the site's resources were
removed.

### Definitions

**Site deletion:** An irreversible, idempotent teardown of resources owned
exclusively by the configured site, followed by removal of its BonesRemote
registration.

**Site registration:** The root-owned BonesRemote directory at
`/root/.config/bonesremote/sites/<site>`. Successful deletion removes it.

**Deletion inventory:** The current safety-validated, secret-free BonesInfra
manifest used to display and execute teardown. It is not persisted remotely.

### Desired Outcome

`bonesdeploy site delete` displays the current site inventory, requires exact
confirmation, removes the site's remote resources, removes its BonesRemote
registration, and reports completion. Rerunning after interruption safely
repeats missing-resource removals and finishes. A later `site setup` follows the
ordinary fresh-setup path.

### Scope

The change removes persisted deletion plans, decommissioning markers,
tombstones, deletion-specific mutation blocking, and setup reactivation. It
retains manifest validation, exact-name confirmation, ordered teardown,
idempotency, and the existing per-site lock while BonesRemote removes its own
registration.

### Constraints

Deletion must preserve local project files and shared server resources. Remote
registration removal must validate the site name, remain within the canonical
BonesRemote sites root, refuse unsafe path kinds, preserve the sites root and
sibling lock, and treat an absent registration as success. BonesRemote
registration is removed only after BonesInfra teardown succeeds.

### Exclusions

The change does not add archival, soft deletion, selective resource retention,
concurrent delete/deploy coordination, or E2E execution during local validation.


## Plan

### Current Behavior

BonesDeploy obtains the current BonesInfra preflight, then sends it to
`bonesremote decommission begin`. BonesRemote persists the plan and marks the
site decommissioning. After BonesInfra teardown, BonesDeploy invokes
`decommission complete` and `verify`, leaving a verified tombstone. Site setup
later invokes `decommission reactivate`. `SiteMutation` rejects decommissioning
and tombstoned sites.

### Intended Behavior

BonesDeploy uses the current validated preflight directly as the deletion plan.
BonesInfra performs the ordered idempotent teardown. Only after teardown
succeeds, BonesDeploy invokes one privileged BonesRemote registration-removal
command. That command validates the site name, acquires the existing site lock,
and safely removes only that site's state directory. Setup has no deletion-state
logic.

### Approach

Replace the four-transition decommission protocol with one registration-removal
operation. Remove deletion fields and types from `SiteState`, remove deletion
checks from `SiteMutation`, and remove setup reactivation. Keep deserialization
tolerant of old state documents long enough for the cleanup command to remove
their containing directory; no persisted values are migrated or acted upon.

BonesDeploy passes the locally generated preflight JSON directly to BonesInfra.
If teardown fails, registration remains and rerunning recomputes and revalidates
the current inventory. If registration cleanup fails, rerunning repeats the
already-idempotent teardown and cleanup.

### Responsibilities And Boundaries

BonesInfra owns inventory validation and remote resource teardown. BonesDeploy
owns confirmation and sequencing. BonesRemote owns safe removal of its state
directory under the existing lock. Normal deployment and setup contain no
deletion-specific state handling.

### Affected Areas

- BonesDeploy site delete/setup orchestration and remote-command helper.
- BonesRemote CLI dispatch and decommission command.
- BonesRemote site state schema and mutation acquisition.
- Decommission, CLI, state cleanup, deletion, and setup tests.
- README, CONTEXT, architecture documentation, and this plan.

### Decisions

The current inventory is recomputed on every retry because teardown operations
are idempotent and the configured project remains the source of site ownership.
The registration directory is removed last so partial teardown remains visible
and retryable. The existing sibling lock protects only registration removal; no
long-lived deletion state or coordinator is introduced.

### Risks

Unsafe state-directory removal could affect other sites or the sites root, so
the command must validate both the site name and final path boundary and reject
symlink targets. Removing deletion fields must not make old state JSON unreadable
before cleanup. A teardown inventory omission can leave resources behind, so the
existing manifest completeness work and focused deletion tests remain required.

### Validation

Tests prove current-plan execution, cleanup only after successful teardown,
missing-registration idempotency, path and symlink refusal, sibling-lock
preservation, old tombstone/decommission-state cleanup, absence of mutation
blocking and setup reactivation, and normal setup after deletion. Run full
non-E2E Rust and Python suites, Clippy, Rustfmt, Ruff, Shfmt, and diff checks.


## Tasks

### Implementation

- [x] Replace BonesRemote begin/complete/verify/reactivate transitions with one
  safe, idempotent site-registration removal command.
- [x] Remove deletion plan/tombstone fields and deletion-specific mutation
  blocking from BonesRemote state.
- [x] Make BonesDeploy execute the current preflight directly and remove the
  registration only after successful BonesInfra teardown.
- [x] Remove setup reactivation and restore ordinary fresh setup behavior.
- [x] Remove obsolete verified-tombstone doctor handling and tests.

### Validation

- [x] Add registration-removal tests for success, missing state, invalid names,
  symlinks, root boundaries, sibling lock preservation, and legacy state.
- [x] Update BonesDeploy, BonesRemote CLI, mutation, setup, and deletion tests for
  the direct teardown flow.
- [x] Run full non-E2E Rust and Python suites plus required lint and format checks.

### Completion

- [x] Update README, CONTEXT, and architecture documentation to remove the
  decommission/tombstone model.
- [x] Review the final diff for obsolete state-machine code, unsafe deletion,
  generated artifact drift, and unrelated changes.

### Completion Notes

Implemented as a direct idempotent teardown. The non-E2E Rust suite, embedded
Python suite, Clippy, rustfmt, Ruff, and shfmt checks pass.


## Idempotent Teardown Clarification

### Trigger

The persisted decommissioning plan and deletion tombstone made successful site
deletion remain visible as an active registration, blocked fresh setup, and
introduced lifecycle states that do not match the command's intended behavior.

### Decision

Site deletion is a direct, idempotent teardown. BonesDeploy validates the
current site inventory, confirms deletion, asks BonesInfra to remove the
site-owned resources, and then asks BonesRemote to remove the site's registration
directory. Missing resources and an already-missing registration are successful
no-ops, so rerunning the command resumes naturally after interruption.

BonesRemote no longer persists deletion plans, decommissioning markers, or
tombstones. It does not block normal mutations based on deletion state, and site
setup does not reactivate deleted state. A deleted site has no registration and
is recreated through the normal setup path.

### Supersedes

This supersedes the persisted deletion plan, decommissioning, tombstone,
mutation-blocking, verification transition, and setup-reactivation design in the
original deletion plan.

### Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` are rewritten to define and track
the idempotent teardown design.
