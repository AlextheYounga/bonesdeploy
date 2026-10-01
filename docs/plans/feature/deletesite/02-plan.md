# Plan

## Current Behavior

BonesDeploy obtains the current BonesInfra preflight, then sends it to
`bonesremote decommission begin`. BonesRemote persists the plan and marks the
site decommissioning. After BonesInfra teardown, BonesDeploy invokes
`decommission complete` and `verify`, leaving a verified tombstone. Site setup
later invokes `decommission reactivate`. `SiteMutation` rejects decommissioning
and tombstoned sites.

## Intended Behavior

BonesDeploy uses the current validated preflight directly as the deletion plan.
BonesInfra performs the ordered idempotent teardown. Only after teardown
succeeds, BonesDeploy invokes one privileged BonesRemote registration-removal
command. That command validates the site name, acquires the existing site lock,
and safely removes only that site's state directory. Setup has no deletion-state
logic.

## Approach

Replace the four-transition decommission protocol with one registration-removal
operation. Remove deletion fields and types from `SiteState`, remove deletion
checks from `SiteMutation`, and remove setup reactivation. Keep deserialization
tolerant of old state documents long enough for the cleanup command to remove
their containing directory; no persisted values are migrated or acted upon.

BonesDeploy passes the locally generated preflight JSON directly to BonesInfra.
If teardown fails, registration remains and rerunning recomputes and revalidates
the current inventory. If registration cleanup fails, rerunning repeats the
already-idempotent teardown and cleanup.

## Responsibilities And Boundaries

BonesInfra owns inventory validation and remote resource teardown. BonesDeploy
owns confirmation and sequencing. BonesRemote owns safe removal of its state
directory under the existing lock. Normal deployment and setup contain no
deletion-specific state handling.

## Affected Areas

- BonesDeploy site delete/setup orchestration and remote-command helper.
- BonesRemote CLI dispatch and decommission command.
- BonesRemote site state schema and mutation acquisition.
- Decommission, CLI, state cleanup, deletion, and setup tests.
- README, CONTEXT, architecture documentation, and this plan.

## Decisions

The current inventory is recomputed on every retry because teardown operations
are idempotent and the configured project remains the source of site ownership.
The registration directory is removed last so partial teardown remains visible
and retryable. The existing sibling lock protects only registration removal; no
long-lived deletion state or coordinator is introduced.

## Risks

Unsafe state-directory removal could affect other sites or the sites root, so
the command must validate both the site name and final path boundary and reject
symlink targets. Removing deletion fields must not make old state JSON unreadable
before cleanup. A teardown inventory omission can leave resources behind, so the
existing manifest completeness work and focused deletion tests remain required.

## Validation

Tests prove current-plan execution, cleanup only after successful teardown,
missing-registration idempotency, path and symlink refusal, sibling-lock
preservation, old tombstone/decommission-state cleanup, absence of mutation
blocking and setup reactivation, and normal setup after deletion. Run full
non-E2E Rust and Python suites, Clippy, Rustfmt, Ruff, Shfmt, and diff checks.
