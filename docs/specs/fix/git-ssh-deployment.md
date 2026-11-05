# Git SSH Deployment

<!-- Merged from fix/gitdeploy -->

## Git Deployment SSH Entry Point

### Request

Run `bonesdeploy deploy` through the `git` SSH identity instead of opening a
root SSH session. Permit `git` to invoke only the exact BonesRemote commands
required for deployment through passwordless, non-interactive sudo.

### Problem

The deployment command currently connects to the server as the configured
privileged SSH user, normally root. This gives routine deployments a root SSH
entry point even though the server already has a dedicated `git` deploy
identity and BonesRemote already constrains deployment behavior.

### Definitions

**Deployment SSH identity:** The Unix account used by `bonesdeploy deploy` to
open the remote SSH session. For this change it is `git`.

**Privileged deployment command:** One complete BonesRemote command executed as
root through `sudo -n`: configuration sync or the existing full deployment
lifecycle.

**Deployment lifecycle:** The existing `bonesremote deploy` implementation,
including its current state, lock, release ownership, activation, rollback, and
cleanup behavior. This change does not split or replace that lifecycle.

### Desired outcome

`bonesdeploy deploy` connects as `git`, runs the exact configuration-sync and
deployment commands through `sudo -n`, and never opens a root SSH session.
BonesRemote continues to execute the existing deployment lifecycle as root,
while build scripts continue to run as `<site>-build` and prepare scripts
continue to run as `<site>`.

### Scope

This change covers the deploy SSH entry point, the two direct sudo command
invocations, the sudoers policy, denial tests, and documentation of the
resulting security boundary.

### Constraints

- Config sync always reads its descriptor from stdin and writes the
  site-derived `/srv/conf/<site>/bones.json` snapshot; deploy always loads that
  snapshot.
- Sudoers permits only direct config sync and deploy commands with one validated
  site argument. It denies arbitrary BonesRemote subcommands, optional extra
  arguments, reordered arguments, and trailing arguments.
- The deployment host requires sudo 1.9.10 or newer for anchored argument
  regular expressions.
- Deployment state, locks, backup credentials, control-plane snapshots, release
  ownership, and lifecycle sequencing remain unchanged.
- Build scripts run only as `<site>-build`.
- Prepare scripts run only as `<site>`.
- Repository-provided scripts never execute as root.
- The shared `git` identity retains its documented single-operator limitation.
- E2E coverage may be added but is not run locally.

### Exclusions

- Splitting the deployment lifecycle into privileged transition subcommands.
- Moving deployment state, locks, snapshots, or backup credentials.
- Changing root password-login policy or other server-hardening settings.
- Changing rollback, cancellation, recovery, backup, or release-management SSH
  entry points.


## Git Deployment SSH Entry Point Plan

### Current behavior

The current branch contains an incomplete first pass. It opens the deployment
session as `git` and sudoes configuration sync, but invokes `bonesremote deploy`
without sudo even though the lifecycle requires root. It also moves snapshots,
state, and locks despite the selected design not requiring those migrations.

Before that first pass, BonesRemote owned the complete root-required lifecycle.
It serializes site mutations with the existing lock, stores deployment state
below the existing root-owned BonesRemote site root, controls the release
namespace, executes build scripts through the dedicated build identity, and
executes prepare scripts through the site runtime identity.

### Intended behavior

The local deploy command opens one SSH session as the standard deploy identity,
`git`. In that session it runs, in order:

```text
sudo -n bonesremote config sync --site <site>
sudo -n bonesremote deploy --site <site>
```

The first command receives the sanitized descriptor on stdin and stores
`/srv/conf/<site>/bones.json`. The second loads that snapshot and runs the
existing lifecycle as root. Sudoers accepts no other operation or argument
shape.

The change is specifically an SSH-entry-point restriction. It does not claim
that the `bonesremote deploy` process has effective UID `git`; sudo executes
that process as root. The security property is that routine deployment does not
open a root SSH session and that the `git` session can elevate only through two
exact command forms.

### Approach

Remove the unnecessary first-pass snapshot, state, lock, and lifecycle ownership
changes. Keep the deploy command's `git` SSH connection and route both operations
directly through sudo. Config sync owns descriptor stdin and deploy loads the
stored snapshot. Use anchored sudoers regexes, add allow and deny tests, and
preserve the provisioned control-plane directory mode. Do not introduce
lifecycle transition commands or move stateful data.

### Responsibilities and boundaries

- `bonesdeploy deploy` owns connection as `git`, descriptor serialization, and
  ordered invocation of the two sudo commands.
- `bonesremote config sync` always reads, validates, and installs the sanitized descriptor
  using the existing root-owned control-plane location.
- `bonesremote deploy` loads that control-plane snapshot and owns the existing
  root-required lifecycle without new orchestration boundaries.
- Existing build and prepare runners retain responsibility for dropping to
  `<site>-build` and `<site>` before repository scripts execute.
- BonesInfra owns rendering and validating the direct sudoers policy.

### Affected areas

- `crates/bonesdeploy/src/commands/deploy.rs`
- `crates/bonesdeploy/src/infra/mod.rs`
- `crates/bonesremote/src/cli/args.rs`
- `crates/bonesremote/src/commands/config.rs`
- `crates/bonesremote/src/commands/deploy/lifecycle.rs`
- `crates/bonesinfra/python/src/bonesinfra/assets/sudoers/bonesdeploy.j2`
- Focused Rust and Python tests for command construction and sudoers rendering
- Documentation describing deployment identities and trust boundaries
- The embedded BonesInfra wheel after the sudoers template changes

### Decisions

- Permit only fixed direct config-sync and deploy commands through sudo instead
  of creating a second lifecycle.
- Keep `bonesremote deploy` root-only because the current lifecycle legitimately
  owns root-controlled state, releases, activation, and service operations.
- Keep configuration sync as the sole stdin consumer and deploy as the sole
  loader of the synchronized descriptor.
- Keep all existing state and filesystem locations; no migration is required.
- Use `sudo -n` so deployment fails immediately rather than prompting when the
  server policy is missing or incorrect.

### Risks

- An overly broad sudoers rule could grant `git` unintended BonesRemote
  authority. Complete anchored command matching and denial tests mitigate this.
- A compromised shared `git` identity can deploy any site allowed by the site
  pattern. This is the existing documented single-operator model, not tenant
  isolation.
- Repository content is processed by a root-owned lifecycle. Existing identity
  boundaries must continue to ensure build and prepare scripts themselves never
  run as root.

### Validation

- Rust tests prove config sync reads stdin without a flag, deploy loads the
  stored snapshot, and local deploy constructs both direct sudo commands.
- Python tests pin the complete rendered sudoers allowlist.
- Denial tests cover arbitrary subcommands, omitted arguments, reordered
  arguments, optional deployment arguments, and trailing arguments.
- Existing lifecycle tests prove build scripts still use `<site>-build`, prepare
  scripts still use `<site>`, and transactional activation behavior is unchanged.
- Run `cargo test --workspace --exclude e2e`, `cargo clippy`, `cargo fmt`,
  `ruff check .`, `ruff format .`, `uv run pytest`, wheel regeneration, and
  `shfmt -w .`. Do not run E2E locally.


## Git Deployment SSH Entry Point Tasks

### Implementation

- [x] Change only `bonesdeploy deploy` to connect through the `git` deploy SSH identity.
- [x] Remove `--config-stdin`; make config sync unconditionally consume stdin and deploy load `/srv/conf/<site>/bones.json`.
- [x] Invoke the two fixed config-sync and deploy commands directly through sudo.
- [x] Replace the wrapper policy with anchored direct BonesRemote sudoers rules.
- [x] Preserve existing deployment state, lock, snapshot, backup-secret, release, and lifecycle behavior.
- [x] Regenerate the embedded BonesInfra wheel after the sudoers template changes.
- [x] Keep control-plane directory permissions at the provisioned `0750` mode and update deployment identity and sudo-boundary documentation.

### Validation

- [x] Add Rust tests proving config sync and deploy have the fixed snapshot contract and local deploy uses both direct sudo commands.
- [x] Add Python tests pinning the complete direct sudoers allowed command set.
- [x] Add denial tests for arbitrary operations, missing/reordered arguments, optional deploy arguments, and trailing arguments.
- [x] Confirm existing tests still prove build scripts run as `<site>-build` and prepare scripts run as `<site>`.
- [x] Run `cargo test --workspace --exclude e2e`.
- [x] Run `cargo clippy`, `cargo fmt`, `ruff check .`, `ruff format .`, `uv run pytest`, regenerate the wheel, and run `shfmt -w .`.
- [x] Review the final diff and confirm no E2E tests were run locally.

### Completion notes

The earlier unprivileged-coordinator and typed-transition approach is
superseded. This change restricts the deployment SSH entry point to `git` while
retaining the existing root-executed BonesRemote lifecycle behind a root-owned
sudoers policy with two anchored direct command forms.

The wheel builder now cleans Python's generated `build/` directory before
packaging, so deleted assets cannot remain in the embedded wheel.

Validation passed: `cargo test --workspace --exclude e2e`, `cargo clippy`,
`cargo fmt`, `ruff check .`, `ruff format .`, `uv run pytest` (494 tests), wheel
regeneration, and `shfmt -w .`. E2E tests were not run.


## Clarification

### Trigger

The earlier plan interpreted “deploy as git” to mean that the
`bonesremote deploy` process and its coordinator must have effective UID
`git`. That interpretation introduced a second lifecycle composed of privileged
transition subcommands, plus new state, lock, and snapshot ownership boundaries.
The requested behavior was clarified to mean that routine deployment must enter
the server through the `git` SSH identity, which then invokes a tightly
allowlisted root-required BonesRemote lifecycle through sudo.

### Decision

Keep the existing root-executed BonesRemote lifecycle intact. Change
`bonesdeploy deploy` to SSH as `git`, then invoke exactly these commands through
`sudo -n`:

```text
bonesremote config sync --site <validated-site> --config-stdin
bonesremote deploy --site <validated-site>
```

Sudoers must anchor each complete argument list and deny all other subcommands,
optional deploy arguments, reordered arguments, and trailing arguments.
Deployment state, locks, snapshots, backup credentials, release ownership, and
lifecycle sequencing stay in their existing root-owned locations. Build and
prepare scripts retain their existing `<site>-build` and `<site>` execution
identities.

This design removes root SSH from routine deployment without creating a second
deployment lifecycle or migrating unrelated state.

### Supersedes

This supersedes the earlier definition of the deploy coordinator as a
`bonesremote deploy` process running with effective UID `git`. It also
supersedes the planned typed lifecycle transitions, git-owned deployment state,
separate lock root, separate snapshot root, and legacy-state migration.

### Effect on the record

`01-idea.md` now defines the deployment SSH identity separately from the
effective identity of the privileged lifecycle and narrows scope to the two sudo
commands. `02-plan.md` now retains the existing lifecycle and root-owned state
while specifying exact command construction, sudoers policy, risks, and tests.
`03-tasks.md` now removes the first-pass state and lifecycle changes, implements
the two-command sudo boundary, and validates both allow and deny cases.


## Clarification

### Trigger

The reviewed implementation retained `--config-stdin` on `bonesremote deploy`
even though the new two-command flow first stores the descriptor with config
sync. That mismatch makes deploy fail. The anchored sudoers regular expressions
also require sudo 1.9.10+, which is not an acceptable host requirement.

### Decision

Remove `--config-stdin` from both public subcommands. `bonesremote config sync
--site <site>` always reads its sanitized descriptor from stdin and atomically
stores `/srv/conf/<site>/bones.json`. `bonesremote deploy --site <site>` always
loads that site-derived snapshot and runs the existing root-owned lifecycle.

Replace version-dependent sudoers regular expressions with a root-owned,
non-writable wrapper. Sudoers permits the wrapper only; it validates one
operation (`config-sync` or `deploy`) and one site identifier, rejects all other
arguments, then executes the corresponding fixed BonesRemote argv. This retains
the two-operation elevation boundary on all supported sudo versions without
requiring a sudo version floor.

Control-plane sync preserves the provisioned `/srv/conf/<site>` mode of `0750`.

### Supersedes

This supersedes the `--config-stdin` argument in the config-sync sudo command,
the deploy lifecycle's stdin descriptor input, and the two anchored sudoers
regular expressions. It supersedes the `0755` control-plane directory mode
applied during config sync.

### Effect on the record

`01-idea.md` now defines the two fixed wrapper operations. `02-plan.md` now
assigns stdin ownership to config sync, snapshot loading to deploy, and argument
validation to the root-owned wrapper. `03-tasks.md` replaces the completed
regex/flag work with the required corrective tasks and validation.


## Clarification

### Trigger

The wrapper introduced to avoid a sudo version floor was rejected. Older sudo
versions are not a supported constraint for this change.

### Decision

Remove the wrapper completely. Routine deploy SSH invokes these direct commands:

```text
sudo -n bonesremote config sync --site <site>
sudo -n bonesremote deploy --site <site>
```

Sudoers uses anchored argument regular expressions for the installed absolute
BonesRemote path. The deployment environment requires sudo 1.9.10 or newer.
Config sync remains the sole stdin consumer and deploy remains the snapshot
loader. The control-plane directory remains mode `0750`.

### Supersedes

This supersedes the wrapper and portable-sudo policy in clarification 05.

### Effect on the record

`01-idea.md`, `02-plan.md`, and `03-tasks.md` restore direct BonesRemote sudo
commands and anchored policy tests, remove wrapper work, and record the sudo
1.9.10 requirement.
