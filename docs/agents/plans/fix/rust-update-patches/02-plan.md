# Plan

## Current behavior

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

## Intended behavior

The local patch registry invokes Rust functions that make the same `.bones`
Git repository and `origin` changes without spawning Bash. Local marker writes
remain atomic and occur only after a local patch succeeds.

The remote patch registry invokes `bonesremote patch apply` through the
existing SSH session. That command applies the matching Rust migration and
writes its own remote marker only after success. It preserves the existing
version-gated ordering and retry semantics for both patch identifiers.

## Approach

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

## Responsibilities and boundaries

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

## Affected areas

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

## Decisions

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

## Risks

- A Rust translation can change Git command arguments or remote URL formatting,
  causing existing `.bones` remotes to be rejected or overwritten incorrectly.
- A remote marker written before migration success would prevent retrying a
  failed server migration.
- The root-owned hook file must stay executable and preserve the existing
  `bonesremote site receive` behavior or config pushes will fail.

## Validation

- Add focused tests proving ordered, version-gated patch selection and local
  `origin` migration behavior without Bash.
- Add focused `bonesremote` tests proving legacy-repository migration,
  canonical bare-repository setup, executable `pre-receive` hook content, and
  remote patch command parsing.
- Run targeted `bonesdeploy` and `bonesremote` tests, `cargo fmt`, `cargo
  clippy`, and `shfmt -w .`; do not run end-to-end tests.
- Review the final diff to confirm all obsolete patch scripts are removed and
  the update documentation describes Rust-owned patches accurately.
