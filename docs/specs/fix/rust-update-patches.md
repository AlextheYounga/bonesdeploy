# Rust Update Patches

<!-- Merged from fix/rust-update-patches -->

## Idea

### Request

Correct the update "patches" approach so it does not depend on Unix shell
scripts. Write local patches in Rust and use the least-cost Rust solution for
remote patches.

### Problem

`bonesdeploy update` embeds Bash scripts and starts `bash` to apply local
patches. The local update workflow therefore cannot run on systems without a
Unix shell. Remote patches also embed and execute shell scripts even though the
installed `bonesremote` binary can own that server-side work.

### Definitions

**Patch:** A version-gated, ordered migration applied during `bonesdeploy`
update. A patch has a stable identifier and is complete only after its
per-project marker has been written. It is not a Git diff or a user deployment
script.

**Local patch:** A patch executed by the local `bonesdeploy` binary against the
project's `.bones` configuration repository. Local patches use the local data
directory for completion markers.

**Remote patch:** A patch executed by the installed `bonesremote` binary on
the deployment server. Remote patches use `/var/lib/bonesdeploy/patches/<site>`
for completion markers.

### Desired outcome

On a machine with Rust's supported platform runtime and Git, `bonesdeploy`
update applies its local configuration-repository migrations without requiring
Bash. Remote configuration-repository migrations retain their existing
observable result, retry behavior, ordering, version gating, and completion
markers while running through `bonesremote` Rust code.

### Scope

- Replace the embedded local Bash patches with Rust implementations in
  `bonesdeploy`.
- Replace the embedded remote Bash patches with a Rust-owned `bonesremote`
  patch command invoked by the existing SSH update flow.
- Preserve the two existing configuration-repository patch identifiers and
  their migration behavior.
- Add regression tests for patch selection and the Rust-owned migration
  behavior.

### Constraints

- Do not require Bash to apply local patches.
- Preserve ordered version selection and per-project, per-scope idempotent
  completion markers.
- Keep remote patch execution within the existing root SSH update flow.
- Keep the generated Git `pre-receive` hook as a shell script because Git runs
  hooks as executable programs.
- Do not run end-to-end tests.

### Exclusions

- Supporting remote operating systems other than the existing Linux server
  target.
- Changing patch versions, identifiers, or the configuration-repository
  migration's externally visible behavior.
- Replacing unrelated shell usage in update, deployment, or Git hooks.


## Plan

### Current behavior

`crates/bonesdeploy/src/commands/update/patches.rs` stores four shell scripts
with `include_str!`, selects the two local and two remote patches by target
version, and uses markers to avoid rerunning completed patches. Local patches
are passed to `bash -s`; remote patches are passed over SSH to a compound Bash
command that manages remote markers.

The local scripts initialize `.bones` as a Git repository when necessary and
add or update its root-owned config-repository `origin`. The remote scripts
migrate a legacy `/home/git/<site>.bones.git` repository to the canonical
`/root/.config/bonesremote/repos/<site>.bones.git` path, create and secure the
bare repository, set its `master` HEAD, and install its `pre-receive` hook.

`update::release::update_remote_from_release` installs the requested
`bonesremote` release before calling `patches::run_remote`. `bonesremote` uses
Clap dispatch into focused modules, and `commands/site.rs` already owns
root-only repository and hook behavior.

### Intended behavior

The local patch registry invokes Rust functions that make the same `.bones`
Git repository and `origin` changes without spawning Bash. Local marker writes
remain atomic and occur only after a local patch succeeds.

The remote patch registry invokes `bonesremote patch apply` through the
existing SSH session. That command applies the matching Rust migration and
writes its own remote marker only after success. It preserves the existing
version-gated ordering and retry semantics for both patch identifiers.

### Approach

Replace script payloads in the `bonesdeploy` patch registry with a Rust
dispatcher. Keep generic Git-at-path process operations in
`crates/bonesdeploy/src/infra/git.rs`. Organize patch ownership beneath
`commands/update/patches/`: local migrations live under `local/`, while
`remote/mod.rs` only sends named patch requests over SSH.

Add a narrow `bonesremote patch apply --site <site> --patch <id>` command.
Its focused module validates the site, applies the canonical config-repository
migration, and atomically writes the remote completion marker. Store the
Git-required `pre-receive` hook in a dedicated hook asset and write it with
the existing permission convention. Call the command from
`bonesdeploy::update::patches` after updating the remote binary.

Delete the obsolete local and remote patch shell-script assets.

### Responsibilities and boundaries

`bonesdeploy::commands::update::patches` owns patch ordering, target version
selection, local marker creation, and orchestration.
`patches::local::config_repo` owns `0001-config-repo`;
`patches::local::root_config_repo` owns `0002-root-config-repo`; and
`patches::remote` owns only the SSH request to the remote binary. Generic Git
operations belong to `infra::git`; local config-repository URL construction
belongs to `patches::local`.

`bonesremote::cli` owns parsing the narrow remote patch command, and
`bonesremote::commands::patch` owns root-only remote migration and remote
marker creation. `bonesremote` retains ownership of remote filesystem and Git
hook changes; `bonesdeploy` only requests the named patch through SSH.

`bonesdeploy-core::paths` remains the source of canonical configuration
repository paths. The existing SSH adapter continues to transport a safely
quoted remote command.

### Affected areas

- `crates/bonesdeploy/src/commands/update/patches/` (`mod.rs`, `local/`,
  `remote.rs`) replacing the single `patches.rs` file
- `crates/bonesdeploy/src/infra/git.rs` for shared Git-at-path operations
- `crates/bonesdeploy/patches/local/*` and `crates/bonesdeploy/patches/remote/*`
  removed
- `crates/bonesremote/src/cli/args.rs`
- `crates/bonesremote/src/cli/dispatch.rs`
- `crates/bonesremote/src/commands/mod.rs`
- New focused `crates/bonesremote/src/commands/patch.rs` module and its
  dedicated `pre-receive` hook asset
- `CONTEXT.md` update-patch documentation

### Decisions

- Remote patches run in `bonesremote`, not BonesInfra, because the remote
  binary is already installed before patches run and directly owns server-side
  filesystem, Git, privilege, and hook behavior. This avoids provisioning a
  Python runtime or a new transport path during updates.
- The remote command writes its own marker so the migration and completion
  state execute under one root-owned binary boundary rather than a compound
  shell command assembled by the local client.
- Both legacy patch identifiers remain registered. They retain existing marker
  compatibility and migration sequencing even though their Rust migration
  implementation is shared.
- The `bonesdeploy` patch code lives in a `patches/` directory module with one
  module per local patch, so future patches add a module instead of growing a
  single file.

### Risks

- A Rust translation can change Git command arguments or remote URL formatting,
  causing existing `.bones` remotes to be rejected or overwritten incorrectly.
- A remote marker written before migration success would prevent retrying a
  failed server migration.
- The root-owned hook file must stay executable and preserve the existing
  `bonesremote site receive` behavior or config pushes will fail.

### Validation

- Add focused tests proving ordered, version-gated patch selection and local
  `origin` migration behavior without Bash.
- Add focused `bonesremote` tests proving legacy-repository migration,
  canonical bare-repository setup, executable `pre-receive` hook content, and
  remote patch command parsing.
- Run targeted `bonesdeploy` and `bonesremote` tests, `cargo fmt`, `cargo
  clippy`, and `shfmt -w .`; do not run end-to-end tests.
- Review the final diff to confirm all obsolete patch scripts are removed and
  the update documentation describes Rust-owned patches accurately.


## Tasks

### Implementation

- [x] Replace `bonesdeploy`'s embedded local shell patches with Rust
  configuration-repository migrations while preserving registry order,
  version gates, remote URL behavior, and atomic local markers.
- [x] Split the single `patches.rs` file into explicit local and remote
  folder modules under `crates/bonesdeploy/src/commands/update/patches/`, with
  generic Git-at-path operations in `crates/bonesdeploy/src/infra/git.rs`.
- [x] Add the root-only `bonesremote patch apply` command that performs the
  configuration-repository migration, writes remote markers after success, and
  installs the executable `pre-receive` hook.
- [x] Route remote update patches through the new `bonesremote` command and
  remove obsolete local and remote patch script assets.
- [x] Update patch workflow documentation for Rust-owned patch behavior.
- [x] Clarify local versus remote patch ownership and move generic Git
  operations to `infra::git`.

### Validation

- [x] Add and run focused tests for local Rust migration behavior, patch
  ordering, and target-version selection.
- [x] Add and run focused tests for remote repository migration, hook creation,
  and patch command parsing.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without running end-to-end tests.

### Completion

- [x] Review the final diff to confirm patch assets are removed, no local Bash
  invocation remains, and documentation reflects the implemented boundaries.

### Completion notes

Implemented the planned Rust patch boundaries without deviations, plus the
documented folder split and explicit local/remote boundaries for `bonesdeploy`
patch code. `cargo test -p
bonesdeploy -p bonesremote` passed (77 and 109 tests), as did `cargo fmt`,
`cargo clippy`, `shfmt -w .`, and `git diff --check`. `tests/ASSERTIONS.md`
is autogenerated and intentionally left untouched. End-to-end tests were not
run by constraint.


## Clarification

### Trigger

The user instructed that keeping all update patch behavior in a single
`patches.rs` file is not a good long-term plan. The patch implementation
should live in a `crates/bonesdeploy/src/commands/update/patches/` folder,
with each patch grouped in its own module.

### Decision

Restructure the `bonesdeploy` side into a directory module at
`crates/bonesdeploy/src/commands/update/patches/` containing:

- `mod.rs` — the patch registry, version ordering and selection, marker
  handling, and local/remote dispatch.
- `config_repo.rs` — the `0001-config-repo` patch: ensure the `.bones`
  directory is a Git repository and its `origin` remote points at the
  configured config repository.
- `root_config_repo.rs` — the `0002-root-config-repo` patch: ensure
  `origin` in `.bones` targets the root-owned config repository.
- `git.rs` — shared Git command and config-repository URL helpers used by
  the patch modules.

Each patch's migration behavior belongs to exactly one module, so a future
patch adds a module instead of growing a single file.

The remote side keeps its single `bonesremote patch apply` command module
because both remote patch identifiers map to one shared configuration
repository migration implemented there.

### Supersedes

The `02-plan.md` affected-areas entry naming
`crates/bonesdeploy/src/commands/update/patches.rs` as the single home of the
local patch implementation.

### Effect on the record

- `01-idea.md`: unchanged; the file layout is an implementation detail below
  the idea's scope.
- `02-plan.md`: "Affected areas" now names the `patches/` folder and its
  four modules; "Responsibilities and boundaries" now assigns each patch
  identifier to its own module.
- `03-tasks.md`: added a task to split the single file into the folder
  module layout, marked with the other completed restructuring work.


## Clarification

### Trigger

The current `patches/git.rs` name and location make generic Git process
operations look like patch-specific behavior. The patch orchestration also
places local migrations and remote SSH dispatch beside each other without a
clear local/remote boundary.

### Decision

Move generic Git-at-path operations into the existing
`crates/bonesdeploy/src/infra/git.rs` module, which owns Git infrastructure.
Keep config-repository URL construction with the local config-repository
patches, where its project configuration meaning is visible.

Organize update patches as explicit local and remote modules:

- `patches/mod.rs` owns the registry, version selection, marker handling, and
  top-level update orchestration.
- `patches/local/mod.rs` owns local patch dispatch and local config-repository
  URL construction.
- `patches/local/config_repo.rs` owns `0001-config-repo`.
- `patches/local/root_config_repo.rs` owns `0002-root-config-repo`.
- `patches/remote/mod.rs` owns only the SSH invocation of the remote
  `bonesremote patch apply` command.

Remote migration implementation remains in `bonesremote`, where remote
filesystem and root privilege operations belong.

### Supersedes

The prior clarification's decision to place shared Git helpers in
`crates/bonesdeploy/src/commands/update/patches/git.rs`, and the corresponding
plan that placed local and remote dispatch together without explicit module
boundaries.

### Effect on the record

- `01-idea.md`: unchanged; this clarifies implementation ownership only.
- `02-plan.md`: updated responsibilities, approach, and affected areas to name
  `infra/git.rs`, `patches/local/`, and `patches/remote/mod.rs`.
- `03-tasks.md`: added and completed the boundary cleanup task.
